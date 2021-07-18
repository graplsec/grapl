from __future__ import annotations

import base64
import hashlib
import inspect
import json
import os
import sys
import traceback
from collections import defaultdict
from logging import Logger
from multiprocessing import Pipe, Process
from multiprocessing.connection import Connection
from multiprocessing.pool import ThreadPool
from pathlib import Path
from typing import (
    TYPE_CHECKING,
    Any,
    Dict,
    Iterable,
    Iterator,
    List,
    Mapping,
    Optional,
    cast,
)

import boto3
from analyzer_executor_lib.redis_cache import EitherCache, construct_redis_client
from analyzer_executor_lib.sqs_types import S3PutRecordDict, SQSMessageBody
from grapl_analyzerlib.analyzer import Analyzer
from grapl_analyzerlib.execution import ExecutionComplete, ExecutionFailed, ExecutionHit
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.base import BaseView
from grapl_analyzerlib.plugin_retriever import load_plugins
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.subgraph_view import SubgraphView
from grapl_common.env_helpers import S3ResourceFactory
from grapl_common.envelope import Envelope, Metadata
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.metrics.metric_reporter import MetricReporter, TagPair

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3ServiceResource


# Set up logger (this is for the whole file, including static methods)
LOGGER = get_module_grapl_logger()

# Set up plugins dir for models
MODEL_PLUGINS_DIR = os.getenv("MODEL_PLUGINS_DIR", "/tmp")
sys.path.insert(0, MODEL_PLUGINS_DIR)

# Ensure plugins dir exists
try:
    directory = Path(MODEL_PLUGINS_DIR + "/model_plugins/")
    directory.mkdir(parents=True, exist_ok=True)
except Exception as e:
    LOGGER.error("Failed to create model plugins directory", e)


def verbose_cast_to_int(input: Optional[str]) -> Optional[int]:
    if not input:
        return None

    try:
        return int(input)
    except (TypeError, ValueError):
        raise ValueError(f"Couldn't cast this env variable into an int: {input}")


class AnalyzerExecutor:

    # constants
    CHUNK_SIZE_RETRY: int = 10
    CHUNK_SIZE_DEFAULT: int = 100

    def __init__(
        self,
        message_cache: EitherCache,
        hit_cache: EitherCache,
        chunk_size: int,
        logger: Logger,
        metric_reporter: MetricReporter,
    ) -> None:
        self.message_cache = message_cache
        self.hit_cache = hit_cache
        self.chunk_size = chunk_size
        self.logger = logger
        self.metric_reporter = metric_reporter

    @classmethod
    def from_env(cls, env: Optional[Mapping[str, str]] = None) -> AnalyzerExecutor:
        env = env or os.environ

        # If we're retrying, change the chunk size
        is_retry = bool(env.get("IS_RETRY", False))
        if is_retry:
            chunk_size = cls.CHUNK_SIZE_RETRY
        else:
            chunk_size = cls.CHUNK_SIZE_DEFAULT

        # Set up message cache
        messagecache_addr = env.get("MESSAGECACHE_ADDR")
        messagecache_port: Optional[int] = verbose_cast_to_int(
            env.get("MESSAGECACHE_PORT")
        )
        message_cache = construct_redis_client(messagecache_addr, messagecache_port)

        # Set up hit cache
        hitcache_addr = env.get("HITCACHE_ADDR")
        hitcache_port: Optional[int] = verbose_cast_to_int(env.get("HITCACHE_PORT"))
        hit_cache = construct_redis_client(hitcache_addr, hitcache_port)

        metric_reporter = MetricReporter.create("analyzer-executor")

        return AnalyzerExecutor(
            message_cache, hit_cache, chunk_size, LOGGER, metric_reporter
        )

    def check_caches(
        self, file_hash: str, msg_id: str, node_key: str, analyzer_name: str
    ) -> bool:
        with self.metric_reporter.histogram_ctx("analyzer-executor.check_caches"):
            if self.check_msg_cache(file_hash, node_key, msg_id):
                self.logger.debug("cache hit - already processed")
                return True

            if self.check_hit_cache(analyzer_name, node_key):
                self.logger.debug("cache hit - already matched")
                return True

            return False

    def to_event_hash(self, components: Iterable[str]) -> str:
        joined = ",".join(components)
        event_hash = hashlib.sha256(joined.encode()).hexdigest()
        return event_hash

    def check_msg_cache(self, file: str, node_key: str, msg_id: str) -> bool:
        event_hash = self.to_event_hash((file, node_key, msg_id))
        return bool(self.message_cache.get(event_hash))

    def update_msg_cache(self, file: str, node_key: str, msg_id: str) -> None:
        event_hash = self.to_event_hash((file, node_key, msg_id))
        self.message_cache.set(event_hash, "1")

    def check_hit_cache(self, file: str, node_key: str) -> bool:
        event_hash = self.to_event_hash((file, node_key))
        return bool(self.hit_cache.get(event_hash))

    def update_hit_cache(self, file: str, node_key: str) -> None:
        event_hash = self.to_event_hash((file, node_key))
        self.hit_cache.set(event_hash, "1")

    async def handle_events(self, events: SQSMessageBody, context: Any) -> None:
        # Parse sns message
        self.logger.debug(f"handling events: {events} context: {context}")

        client = GraphClient()

        s3 = S3ResourceFactory(boto3).from_env()

        load_plugins(
            os.environ["DEPLOYMENT_NAME"],
            s3.meta.client,
            os.path.abspath(MODEL_PLUGINS_DIR),
        )

        for event in events["Records"]:
            data = parse_s3_event(s3, event)

            envelope = Envelope.from_proto_bytes(data)
            # Keep in mind that, today, we don't communicate this message via protobuf but instead
            # we use JSON. It would be worth changing this in another issue.
            message = json.loads(envelope.inner_message)

            LOGGER.info(f'Executing Analyzer: {message["key"]}')

            with self.metric_reporter.histogram_ctx(
                "analyzer-executor.download_s3_file"
            ):
                analyzer = download_s3_file(
                    s3,
                    f"{os.environ['DEPLOYMENT_NAME']}-analyzers-bucket",
                    message["key"],
                ).decode("utf8")
            analyzer_name = message["key"].split("/")[-2]

            subgraph = SubgraphView.from_proto(client, bytes(message["subgraph"]))

            # TODO: Validate signature of S3 file
            LOGGER.info(f"event {event} {envelope.metadata}")
            rx: Connection
            tx: Connection
            rx, tx = Pipe(duplex=False)
            p = Process(
                target=self.execute_file,
                args=(analyzer_name, analyzer, subgraph, tx, "", self.chunk_size),
            )

            p.start()

            for exec_hit in self.poll_process(rx=rx, analyzer_name=analyzer_name):
                with self.metric_reporter.histogram_ctx(
                    "analyzer-executor.emit_event.ms",
                    (TagPair("analyzer_name", exec_hit.analyzer_name),),
                ):
                    emit_event(s3, exec_hit, envelope.metadata)
                self.update_msg_cache(analyzer, exec_hit.root_node_key, message["key"])
                self.update_hit_cache(analyzer_name, exec_hit.root_node_key)

            p.join()

    def poll_process(
        self,
        rx: Connection,
        analyzer_name: str,
    ) -> Iterator[ExecutionHit]:
        """
        Keep polling the spawned Process, and yield any ExecutionHits.
        (This will probably disappear if Analyzers move to Docker images.)
        """
        t = 0

        while True:
            p_res = rx.poll(timeout=5)
            if not p_res:
                t += 1
                LOGGER.info(
                    f"Analyzer {analyzer_name} polled for for {t * 5} seconds without result"
                )
                continue

            result: Optional[Any] = rx.recv()
            if isinstance(result, ExecutionComplete):
                self.logger.info(f"Analyzer {analyzer_name} execution complete")
                return

            # emit any hits to an S3 bucket
            if isinstance(result, ExecutionHit):
                self.logger.info(
                    f"Analyzer {analyzer_name} emitting event for:"
                    f"{result.analyzer_name} {result.root_node_key}"
                )
                yield result

            assert not isinstance(
                result, ExecutionFailed
            ), f"Analyzer {analyzer_name} failed."

    def exec_analyzers(
        self,
        dg_client: GraphClient,
        file: str,
        msg_id: str,
        nodes: List[BaseView],
        analyzers: Dict[str, Analyzer],
        sender: Any,
    ) -> None:
        if not analyzers:
            self.logger.warning("Received empty dict of analyzers")
            return

        if not nodes:
            self.logger.warning("Received empty array of nodes")

        for node in nodes:
            querymap: Dict[str, List[Queryable]] = defaultdict(list)

            for an_name, analyzer in analyzers.items():
                if self.check_caches(file, msg_id, node.node_key, an_name):
                    continue

                queries = analyzer.get_queries()
                if isinstance(queries, list) or isinstance(queries, tuple):
                    querymap[an_name].extend(queries)
                else:
                    querymap[an_name].append(queries)

            for an_name, queries in querymap.items():
                analyzer = analyzers[an_name]

                for query in queries:
                    # TODO: Whether it was a hit or not is a good Tag
                    tags = (TagPair("analyzer_name", an_name),)
                    with self.metric_reporter.histogram_ctx(
                        "analyzer-executor.query_first.ms", tags
                    ):
                        response = query.query_first(
                            dg_client, contains_node_key=node.node_key
                        )
                    if response:
                        self.logger.debug(
                            f"Analyzer '{an_name}' received a hit, executing on_response()"
                        )
                        with self.metric_reporter.histogram_ctx(
                            "analyzer-executor.on_response.ms", tags
                        ):
                            analyzer.on_response(response, sender)

    def execute_file(
        self,
        name: str,
        file: str,
        graph: SubgraphView,
        sender: Connection,
        msg_id: str,
        chunk_size: int,
    ) -> None:
        try:
            pool = ThreadPool(processes=4)

            exec(file, globals())
            client = GraphClient()

            analyzers = get_analyzer_objects(client)
            if not analyzers:
                self.logger.warning(f"Got no analyzers for file: {name}")

            self.logger.info(f"Executing analyzers: {[an for an in analyzers.keys()]}")

            for nodes in chunker([n for n in graph.node_iter()], chunk_size):
                self.logger.info(f"Querying {len(nodes)} nodes")

                def exec_analyzer(
                    nodes: List[BaseView], sender: Connection
                ) -> List[BaseView]:
                    try:
                        self.exec_analyzers(
                            client, file, msg_id, nodes, analyzers, sender
                        )

                        return nodes
                    except Exception as e:
                        self.logger.error(traceback.format_exc())
                        self.logger.error(
                            f"Execution of {name} failed with {e} {e.args}"
                        )
                        sender.send(ExecutionFailed())
                        raise

                pool.apply_async(exec_analyzer, args=(nodes, sender))

            pool.close()

            pool.join()

            sender.send(ExecutionComplete())

        except Exception as e:
            self.logger.error(traceback.format_exc())
            self.logger.error(f"Execution of {name} failed with {e} {e.args}")
            sender.send(ExecutionFailed())
            raise


def parse_s3_event(s3: S3ServiceResource, event: S3PutRecordDict) -> bytes:
    try:
        bucket = event["s3"]["bucket"]["name"]
        key = event["s3"]["object"]["key"]
    except KeyError:
        LOGGER.error("Could not parse s3 event: {}", exc_info=True)
        raise
    return download_s3_file(s3, bucket, key)


def download_s3_file(s3: S3ServiceResource, bucket: str, key: str) -> bytes:
    obj = s3.Object(bucket, key)
    return cast(bytes, obj.get()["Body"].read())


def is_analyzer(analyzer_name: str, analyzer_cls: type) -> bool:
    if analyzer_name == "Analyzer":  # This is the base class
        return False
    return (
        hasattr(analyzer_cls, "get_queries")
        and hasattr(analyzer_cls, "build")
        and hasattr(analyzer_cls, "on_response")
    )


def get_analyzer_objects(dgraph_client: GraphClient) -> Dict[str, Analyzer]:
    clsmembers = inspect.getmembers(sys.modules[__name__], inspect.isclass)
    return {
        an[0]: an[1].build(dgraph_client)
        for an in clsmembers
        if is_analyzer(an[0], an[1])
    }


def chunker(seq: List[BaseView], size: int) -> List[List[BaseView]]:
    return [seq[pos : pos + size] for pos in range(0, len(seq), size)]


def emit_event(s3: S3ServiceResource, event: ExecutionHit, metadata: Metadata) -> None:
    LOGGER.info(f"emitting event for: {event.analyzer_name, event.nodes}")

    meta_dict = {
        "trace_id": str(metadata.get_trace_id()),
    }

    event_s = json.dumps(
        {
            "nodes": json.loads(event.nodes),
            "edges": json.loads(event.edges),
            "analyzer_name": event.analyzer_name,
            "risk_score": event.risk_score,
            "lenses": event.lenses,
            "risky_node_keys": event.risky_node_keys,
            "metadata": meta_dict,
        }
    )
    event_hash = hashlib.sha256(event_s.encode())
    key = base64.urlsafe_b64encode(event_hash.digest()).decode("utf-8")

    obj = s3.Object(
        f"{os.environ['DEPLOYMENT_NAME']}-analyzer-matched-subgraphs-bucket", key
    )
    obj.put(Body=event_s.encode("utf-8"))

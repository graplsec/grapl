from __future__ import annotations

import json
import os
import sys
import time
from typing import Any, Dict, List, Union, cast

import boto3
import pydgraph
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    EdgeT,
    PropPrimitive,
    PropType,
)
from grapl_analyzerlib.nodes.base import BaseSchema
from grapl_analyzerlib.prelude import (
    AssetSchema,
    FileSchema,
    IpAddressSchema,
    IpConnectionSchema,
    IpPortSchema,
    LensSchema,
    NetworkConnectionSchema,
    ProcessInboundConnectionSchema,
    ProcessOutboundConnectionSchema,
    ProcessSchema,
    RiskSchema,
)
from grapl_analyzerlib.provision import provision_common
from grapl_common.env_helpers import DynamoDBResourceFactory
from grapl_common.grapl_logger import get_module_grapl_logger

LOGGER = get_module_grapl_logger(default_log_level="INFO")


def set_schema(client: GraphClient, schema: str) -> None:
    op = pydgraph.Operation(schema=schema)
    client.alter(op)


def drop_all(client: GraphClient) -> None:
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)


def format_schemas(schema_defs: List[BaseSchema]) -> str:
    schemas = "\n\n".join([schema.generate_schema() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join(
        ["  # Type Definitions", types, "\n  # Schema Definitions", schemas]
    )


def query_dgraph_predicate(client: GraphClient, predicate_name: str) -> Dict[str, Any]:
    query = f"""
        schema(pred: {predicate_name}) {{  }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)["schema"][0]
    finally:
        txn.discard()

    return cast(Dict[str, Any], res)


def meta_into_edge(schema: BaseSchema, predicate_meta: Dict[str, Any]) -> EdgeT:
    if predicate_meta.get("list"):
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToMany)
    else:
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToOne)


def meta_into_property(predicate_meta: Dict[str, Any]) -> PropType:
    is_set = predicate_meta["list"]
    type_name = predicate_meta["type"]
    primitives = {
        "string": PropPrimitive.Str,
        "int": PropPrimitive.Int,
        "bool": PropPrimitive.Bool,
    }

    return PropType(
        primitives[type_name], is_set, index=predicate_meta.get("index", [])
    )


def meta_into_predicate(
    schema: BaseSchema, predicate_meta: Dict[str, Any]
) -> Union[EdgeT, PropType]:
    try:
        if predicate_meta["type"] == "uid":
            return meta_into_edge(schema, predicate_meta)
        else:
            return meta_into_property(predicate_meta)
    except Exception as e:
        LOGGER.error(f"Failed to convert meta to predicate: {predicate_meta} {e}")
        raise e


def query_dgraph_type(client: GraphClient, type_name: str) -> List[Dict[str, Any]]:
    query = f"""
        schema(type: {type_name}) {{ type }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)
    finally:
        txn.discard()

    if not res:
        return []
    if not res.get("types"):
        return []

    res = res["types"][0]["fields"]
    predicate_names = []
    for pred in res:
        predicate_names.append(pred["name"])

    predicate_metas = []
    for predicate_name in predicate_names:
        predicate_metas.append(query_dgraph_predicate(client, predicate_name))

    return predicate_metas


def extend_schema(graph_client: GraphClient, schema: BaseSchema) -> None:
    predicate_metas = query_dgraph_type(graph_client, schema.self_type())

    for predicate_meta in predicate_metas:
        predicate = meta_into_predicate(schema, predicate_meta)
        if isinstance(predicate, PropType):
            schema.add_property(predicate_meta["predicate"], predicate)
        else:
            schema.add_edge(predicate_meta["predicate"], predicate, "")


def provision_master_graph(
    master_graph_client: GraphClient, schemas: List["BaseSchema"]
) -> None:
    mg_schema_str = format_schemas(schemas)
    set_schema(master_graph_client, mg_schema_str)


def provision_mg(mclient: GraphClient) -> None:
    drop_all(mclient)

    schemas = [
        AssetSchema(),
        ProcessSchema(),
        FileSchema(),
        IpConnectionSchema(),
        IpAddressSchema(),
        IpPortSchema(),
        NetworkConnectionSchema(),
        ProcessInboundConnectionSchema(),
        ProcessOutboundConnectionSchema(),
        RiskSchema(),
        LensSchema(),
    ]

    for schema in schemas:
        schema.init_reverse()

    for schema in schemas:
        extend_schema(mclient, schema)

    provision_master_graph(mclient, schemas)

    deployment_name = "local-grapl"  # TODO replace?
    dynamodb = DynamoDBResourceFactory(boto3).from_env()
    schema_table = provision_common.get_schema_table(
        dynamodb, deployment_name=deployment_name
    )
    schema_properties_table = provision_common.get_schema_properties_table(
        dynamodb, deployment_name=deployment_name
    )

    for schema in schemas:
        provision_common.store_schema(schema_table, schema)
        provision_common.store_schema_properties(schema_properties_table, schema)


DEPLOYMENT_NAME = "local-grapl"


def validate_environment() -> None:
    """Ensures that the required environment variables are present in the environment.

    Other code actually reads the variables later.
    """
    required = [
        "AWS_REGION",
        "DYNAMODB_ACCESS_KEY_ID",
        "DYNAMODB_ACCESS_KEY_SECRET",
        "DYNAMODB_ENDPOINT",
    ]

    missing = [var for var in required if var not in os.environ]

    if missing:
        print(
            f"The following environment variables are required, but are not present: {missing}"
        )
        sys.exit(1)


if __name__ == "__main__":
    validate_environment()

    time.sleep(5)
    graph_client = GraphClient()

    LOGGER.debug("Provisioning graph database")

    for i in range(0, 150):
        try:
            drop_all(graph_client)
            break
        except Exception as e:
            time.sleep(2)
            if i > 20:
                LOGGER.error("Failed to drop", e)

    mg_succ = False

    LOGGER.info("Starting to provision master graph")
    for i in range(0, 150):
        try:
            if not mg_succ:
                time.sleep(1)
                provision_mg(
                    graph_client,
                )
                mg_succ = True
                LOGGER.info("Provisioned master graph")
                break
        except Exception as e:
            if i <= 10:
                LOGGER.error(f"Provision failure {i}/150:", e)
            elif 10 < i <= 140:
                LOGGER.error(f"Provision failure {i}/150")
            else:
                raise e

    LOGGER.info("Completed provisioning")

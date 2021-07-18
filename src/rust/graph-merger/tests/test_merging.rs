#[cfg(feature = "integration")]
pub mod test {
    use std::{
        collections::HashMap,
        sync::{
            Arc,
            Once,
        },
    };

    use dgraph_query_lib::{
        condition::{
            Condition,
            ConditionValue,
        },
        predicate::{
            Field,
            Predicate,
        },
        queryblock::QueryBlockType,
        schema::{
            Indexing,
            PredicateDefinition,
            PredicateType,
            Schema,
            SchemaDefinition,
        },
        EdgeBuilder,
        QueryBlockBuilder,
        QueryBuilder,
        ToQueryString,
    };
    use dgraph_tonic::{
        Client as DgraphClient,
        Query,
    };
    use graph_merger_lib::upserter::GraphMergeHelper;
    use rust_proto::*;

    async fn query_for_uid(dgraph_client: Arc<DgraphClient>, node_key: &str) -> u64 {
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                "node_key".to_string(),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![Predicate::Field(Field::new("uid"))])
            .first(1)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let m = m.into_iter().next().unwrap().1;
        debug_assert!((m.len() == 1) || (m.len() == 0));

        let uid = &m[0]["uid"][2..];
        let uid = u64::from_str_radix(uid, 16).expect("uid is not valid hex");
        uid
    }

    async fn query_for_edge(
        dgraph_client: Arc<DgraphClient>,
        from_uid: u64,
        edge_name: &str,
    ) -> serde_json::Value {
        let edge = Predicate::Edge(
            EdgeBuilder::default()
                .name(edge_name.to_string())
                .predicates(vec![
                    Predicate::Field(Field::new("uid")),
                    Predicate::Field(Field::new("dgraph.type").alias("dgraph_type")),
                ])
                .build()
                .unwrap(),
        );

        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::uid(&from_uid.to_string()))
            .predicates(vec![Predicate::Field(Field::new("uid")), edge])
            .first(1)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        serde_json::from_slice(&response.json).expect("response failed to parse")
    }

    fn init_test_env() {
        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);

        static START: Once = Once::new();
        START.call_once(|| {
            let schema = Schema::new()
                .add_definition(
                    SchemaDefinition::new("ExampleNode")
                        .add_predicate(
                            PredicateDefinition::new("example_id", PredicateType::INT)
                                .add_index(Indexing::INT),
                        )
                        .add_predicate(
                            PredicateDefinition::new("node_key", PredicateType::String)
                                .add_index(Indexing::EXACT)
                                .upsert(),
                        )
                        .add_predicate(
                            PredicateDefinition::new("example_name", PredicateType::String)
                                .add_index(Indexing::TRIGRAM),
                        )
                        .add_predicate(PredicateDefinition::new(
                            "to_many_edge",
                            PredicateType::UIDArray,
                        ))
                        .add_predicate(PredicateDefinition::new(
                            "to_single_edge",
                            PredicateType::UID,
                        )),
                )
                .to_string();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("failed to init runtime");
                let mg_alpha = grapl_config::mg_alphas()
                    .pop()
                    .expect("Dgraph Alpha not specified.");

                rt.block_on(async {
                    let dgraph_client =
                        DgraphClient::new(mg_alpha).expect("Failed to create dgraph client.");

                    dgraph_client
                        .alter(dgraph_tonic::Operation {
                            drop_all: true,
                            ..Default::default()
                        })
                        .await
                        .expect("alter failed");

                    dgraph_client
                        .alter(dgraph_tonic::Operation {
                            schema,
                            ..Default::default()
                        })
                        .await
                        .expect("alter failed");
                });
            })
            .join()
            .expect("provision failed");
        });
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_upsert_edge_and_retrieve() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let mg_alpha = grapl_config::mg_alphas()
            .pop()
            .expect("Dgraph Alpha not specified.");

        let mut identified_graph = IdentifiedGraph::new();
        let mut merged_graph = MergedGraph::new();
        let dgraph_client = DgraphClient::new(mg_alpha).expect("Failed to create dgraph client.");
        let dgraph_client = std::sync::Arc::new(dgraph_client);
        let mut properties = HashMap::new();
        properties.insert(
            "example_name".to_string(),
            ImmutableStrProp {
                prop: "foobar".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            node_key: "example-node-key".to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        let mut properties = HashMap::new();
        properties.insert(
            "example_name".to_string(),
            ImmutableStrProp {
                prop: "baz".to_string(),
            }
            .into(),
        );

        let n1 = IdentifiedNode {
            node_key: "someother-node-key".to_string(),
            node_type: "Process".to_string(),
            properties,
        };

        identified_graph.add_node(n0);
        identified_graph.add_node(n1);

        identified_graph.add_edge(
            "to_many_edge".to_string(),
            "example-node-key".to_string(),
            "someother-node-key".to_string(),
        );

        identified_graph.add_edge(
            "to_single_edge".to_string(),
            "someother-node-key".to_string(),
            "example-node-key".to_string(),
        );

        GraphMergeHelper {}
            .upsert_into(dgraph_client.clone(), &identified_graph, &mut merged_graph)
            .await;

        let node_uid_0 = query_for_uid(dgraph_client.clone(), "example-node-key").await;
        let node_uid_1 = query_for_uid(dgraph_client.clone(), "someother-node-key").await;
        assert_ne!(node_uid_0, node_uid_1);
        assert_ne!(node_uid_0, 0);
        assert_ne!(node_uid_1, 0);

        let to_many_res = query_for_edge(dgraph_client.clone(), node_uid_0, "to_many_edge").await;

        let to_single_res =
            query_for_edge(dgraph_client.clone(), node_uid_1, "to_single_edge").await;

        let to_many_res = to_many_res
            .as_object()
            .expect("to_many_res.as_object")
            .values()
            .next()
            .expect("to_many_res empty array");
        let to_single_res = to_single_res
            .as_object()
            .expect("to_single_res.as_object")
            .values()
            .next()
            .expect("to_single_res empty array");

        let tm_from = to_many_res[0]["uid"].as_str().expect("tm_from");
        let tm_to = to_many_res[0]["to_many_edge"][0]["uid"]
            .as_str()
            .expect("tm_to");

        let ts_from = to_single_res[0]["uid"].as_str().expect("ts_from");
        let ts_to = to_single_res[0]["to_single_edge"]["uid"]
            .as_str()
            .expect("ts_to");

        assert_eq!(tm_from, ts_to);
        assert_eq!(tm_to, ts_from);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_upsert_idempotency() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let mg_alpha = grapl_config::mg_alphas()
            .pop()
            .expect("Dgraph Alpha not specified.");

        let dgraph_client = DgraphClient::new(mg_alpha).expect("Failed to create dgraph client.");
        let dgraph_client = std::sync::Arc::new(dgraph_client);

        let node_key = "test_upsert_idempotency-example-node-key";
        let mut properties = HashMap::new();
        properties.insert(
            "example_name".to_string(),
            ImmutableStrProp {
                prop: "foobar".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };

        let upsert_futs: Vec<_> = (0..10)
            .map(|_| {
                let dgraph_client = dgraph_client.clone();
                let n0 = n0.clone();
                async move {
                    let mut identified_graph = IdentifiedGraph::new();
                    identified_graph.add_node(n0);
                    let mut merged_graph = MergedGraph::new();

                    GraphMergeHelper {}
                        .upsert_into(dgraph_client.clone(), &identified_graph, &mut merged_graph)
                        .await;
                    merged_graph
                }
            })
            .collect();

        let mut merged_graphs = Vec::with_capacity(upsert_futs.len());
        for upsert_fut in upsert_futs.into_iter() {
            merged_graphs.push(upsert_fut.await);
        }

        for merged_graph in merged_graphs {
            assert_eq!(merged_graph.nodes.len(), 1);
        }

        // If we query for multiple nodes by node_key we should only ever receive one
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                "node_key".to_string(),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![Predicate::Field(Field::new("uid"))])
            .first(2)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let m = m.into_iter().next().unwrap().1;
        debug_assert_eq!(m.len(), 1);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_upsert_multifield() -> Result<(), Box<dyn std::error::Error>> {
        init_test_env();
        let mg_alpha = grapl_config::mg_alphas()
            .pop()
            .expect("Dgraph Alpha not specified.");

        let dgraph_client = DgraphClient::new(mg_alpha).expect("Failed to create dgraph client.");
        let dgraph_client = std::sync::Arc::new(dgraph_client);

        let node_key = "test_upsert_multifield-example-node-key";
        let mut properties = HashMap::new();
        properties.insert(
            "example_name".to_string(),
            ImmutableStrProp {
                prop: "test_upsert_multifield".to_string(),
            }
            .into(),
        );
        let n0 = IdentifiedNode {
            node_key: node_key.to_string(),
            node_type: "ExampleNode".to_string(),
            properties,
        };
        let mut identified_graph = IdentifiedGraph::new();
        identified_graph.add_node(n0);
        let mut merged_graph = MergedGraph::new();

        GraphMergeHelper {}
            .upsert_into(dgraph_client.clone(), &identified_graph, &mut merged_graph)
            .await;

        // If we query for multiple nodes by node_key we should only ever receive one
        let query_block = QueryBlockBuilder::default()
            .query_type(QueryBlockType::query())
            .root_filter(Condition::EQ(
                "node_key".to_string(),
                ConditionValue::string(node_key),
            ))
            .predicates(vec![
                Predicate::Field(Field::new("uid")),
                Predicate::Field(Field::new("example_name")),
            ])
            // .first(2)
            .build()
            .unwrap();

        let query = QueryBuilder::default()
            .query_blocks(vec![query_block])
            .build()
            .unwrap();

        let mut txn = dgraph_client.new_read_only_txn();
        let response = txn
            .query(query.to_query_string())
            .await
            .expect("query failed");

        let m: HashMap<String, Vec<HashMap<String, String>>> =
            serde_json::from_slice(&response.json).expect("response failed to parse");
        let mut m = m.into_iter().next().unwrap().1;
        debug_assert_eq!(m.len(), 1);
        let mut m = m.remove(0);
        let _uid = m.remove("uid").expect("uid");
        let example_name = m.remove("example_name").expect("example_name");
        assert!(m.is_empty());
        assert_eq!(example_name, "test_upsert_multifield");
        Ok(())
    }
}

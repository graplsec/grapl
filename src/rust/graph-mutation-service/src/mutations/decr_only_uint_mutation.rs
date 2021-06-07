use grapl_graph_descriptions::DecrementOnlyUintProp;

use crate::mutations::{
    escape::Escaped,
    upsert_helpers::{
        gen_mutations,
        gen_query,
    },
    QueryInput,
    UpsertGenerator,
};

#[derive(Default)]
pub struct DecrementOnlyUintUpsertGenerator {
    query_buffer: String,
    mutations: Vec<dgraph_tonic::Mutation>,
}

impl UpsertGenerator for DecrementOnlyUintUpsertGenerator {
    type Input = DecrementOnlyUintProp;
    fn generate_upserts(
        &mut self,
        creation_query: &QueryInput<'_>,
        predicate_name: &str,
        value: &Self::Input,
    ) -> (&str, &[dgraph_tonic::Mutation]) {
        let DecrementOnlyUintProp { prop: ref value } = value;
        let value = Escaped::from(value);
        let query_suffix = format!(
            "{}_{}_{}",
            &creation_query.unique_id, &creation_query.node_id, &creation_query.predicate_id
        );
        let (set_query_name, cmp_query_name) = gen_query(
            &mut self.query_buffer,
            "gt",
            &creation_query.creation_query_name,
            &query_suffix,
            predicate_name,
            &format!("{}_{}", predicate_name, creation_query.node_id),
            &value,
        );

        gen_mutations(
            &mut self.mutations,
            &creation_query.creation_query_name,
            &set_query_name,
            &cmp_query_name,
            &predicate_name,
            &value,
        );

        (&self.query_buffer, &self.mutations)
    }
}

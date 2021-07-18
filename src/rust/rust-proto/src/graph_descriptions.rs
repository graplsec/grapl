pub use node_property::Property;
use node_property::Property::{
    DecrementOnlyInt as ProtoDecrementOnlyIntProp,
    DecrementOnlyUint as ProtoDecrementOnlyUintProp,
    ImmutableInt as ProtoImmutableIntProp,
    ImmutableStr as ProtoImmutableStrProp,
    ImmutableUint as ProtoImmutableUintProp,
    IncrementOnlyInt as ProtoIncrementOnlyIntProp,
    IncrementOnlyUint as ProtoIncrementOnlyUintProp,
};

pub use crate::graplinc::grapl::api::graph::v1beta1::*;
use crate::services::ServiceMessage;

// A helper macro to generate `From` impl boilerplate.
macro_rules ! impl_from_for_unit {
    ($into_t:ty, $field:tt, $from_t:ty) => {
        impl From<$from_t> for $into_t
        {
            fn from(p: $from_t) -> Self {
                let p = p.to_owned().into();
                Self {$field: p}
            }
        }
    };
    ($into_t:ty, $field:tt, $head:ty, $($tail:ty),*) => {
        impl_from_for_unit!($into_t, $field, $head);
        impl_from_for_unit!($into_t, $field, $($tail),*);
    };
}

macro_rules ! extra_assert {
    ($x:expr) => {
        #[cfg(feature = "extra_assertions")]
        {
            $x
        }
    };
    ($x:expr, $($tail:ty),*) => {
        extra_assert!($x);
        extra_assert!($($tail), *);
    }
}

impl EdgeList {
    pub fn into_vec(self) -> Vec<Edge> {
        let Self { edges } = self;
        edges
    }
}

impl ServiceMessage for GraphDescription {
    const TYPE_NAME: &'static str = "graplinc.grapl.api.graph.v1beta1.GraphDescription";
}

impl GraphDescription {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: impl Into<NodeDescription>) {
        let node = node.into();
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        to_node_key: impl Into<String>,
    ) {
        let from_node_key = from_node_key.into();
        let to_node_key = to_node_key.into();
        let edge_name = edge_name.into();

        assert_ne!(from_node_key, to_node_key);

        let edge = Edge {
            from_node_key: from_node_key.clone(),
            to_node_key,
            edge_name,
        };

        let edge_list: &mut Vec<Edge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| EdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(
                    edge.edge_name.clone(),
                    edge.from_node_key.clone(),
                    edge.to_node_key.clone(),
                );
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl IdentifiedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: IdentifiedNode) {
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        to_node_key: impl Into<String>,
    ) {
        let from_node_key = from_node_key.into();
        let to_node_key = to_node_key.into();
        assert_ne!(from_node_key, to_node_key);

        let edge_name = edge_name.into();
        let edge = Edge {
            from_node_key: from_node_key.clone(),
            to_node_key,
            edge_name,
        };

        let edge_list: &mut Vec<Edge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| EdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(
                    edge.edge_name.clone(),
                    edge.from_node_key.clone(),
                    edge.to_node_key.clone(),
                );
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl ServiceMessage for IdentifiedGraph {
    const TYPE_NAME: &'static str = "graplinc.grapl.api.graph.v1beta1.IdentifiedGraph";
}

impl MergedGraph {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            edges: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: MergedNode) {
        match self.nodes.get_mut(&node.node_key) {
            Some(n) => n.merge(&node),
            None => {
                self.nodes.insert(node.clone_node_key(), node);
            }
        };
    }

    pub fn add_merged_edge(&mut self, edge: MergedEdge) {
        let from_node_key = edge.from_node_key.clone();
        let edge_list: &mut Vec<MergedEdge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| MergedEdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn add_edge(
        &mut self,
        edge_name: impl Into<String>,
        from_node_key: impl Into<String>,
        from_uid: impl Into<String>,
        to_node_key: impl Into<String>,
        to_uid: impl Into<String>,
    ) {
        let edge_name = edge_name.into();
        let from_node_key = from_node_key.into();
        let from_uid = from_uid.into();
        let to_node_key = to_node_key.into();
        let to_uid = to_uid.into();
        assert_ne!(from_node_key, to_node_key);
        assert_ne!(from_uid, to_uid);
        let edge = MergedEdge {
            from_node_key: from_node_key.clone(),
            from_uid,
            to_node_key,
            to_uid,
            edge_name,
        };

        let edge_list: &mut Vec<MergedEdge> = &mut self
            .edges
            .entry(from_node_key)
            .or_insert_with(|| MergedEdgeList {
                edges: Vec::with_capacity(1),
            })
            .edges;
        edge_list.push(edge);
    }

    pub fn merge(&mut self, other: &Self) {
        for (node_key, other_node) in other.nodes.iter() {
            match self.nodes.get_mut(node_key) {
                Some(n) => n.merge(other_node),
                None => {
                    self.nodes.insert(node_key.clone(), other_node.clone());
                }
            };
        }

        for edge_list in other.edges.values() {
            for edge in edge_list.edges.iter() {
                self.add_edge(
                    edge.edge_name.clone(),
                    edge.from_node_key.clone(),
                    edge.from_uid.clone(),
                    edge.to_node_key.clone(),
                    edge.to_uid.clone(),
                );
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }
}

impl NodeDescription {
    pub fn merge(&mut self, other: &Self) {
        extra_assert!(debug_assert_eq!(self.node_type, other.node_type));
        extra_assert!(debug_assert_eq!(self.node_key, other.node_key));
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }
    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

impl ServiceMessage for MergedGraph {
    const TYPE_NAME: &'static str = "graplinc.grapl.api.graph.v1beta1.MergedGraph";
}

impl IdentifiedNode {
    pub fn merge(&mut self, other: &Self) {
        extra_assert!(debug_assert_eq!(self.node_type, other.node_type));
        extra_assert!(debug_assert_eq!(self.node_key, other.node_key));
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }

    pub fn get_cache_identities_for_predicates(&self) -> Vec<Vec<u8>> {
        let mut predicate_cache_identities = Vec::with_capacity(self.properties.len());

        for (key, prop) in &self.properties {
            let prop_value = prop
                .property
                .as_ref()
                .map(Property::to_string)
                .unwrap_or_else(|| panic!("Invalid property on DynamicNode: {}", self.node_key));

            predicate_cache_identities.push(format!("{}:{}:{}", &self.node_key, key, prop_value));
        }

        predicate_cache_identities
            .into_iter()
            .map(|item| item.into_bytes())
            .collect()
    }
}

impl NodeProperty {
    pub fn merge(&mut self, other: &Self) {
        match (self.property.as_mut(), other.property.as_ref()) {
            (Some(p), Some(op)) => {
                p.merge_property(op);
            }
            (None, Some(op)) => {
                debug_assert!(false, "Unhandled property merge, self is None: {:?}", op);
                tracing::warn!("Unhandled property merge, self is None: {:?}", op);
            }
            (Some(p), None) => {
                debug_assert!(false, "Unhandled property merge, other is None: {:?}", p);
                tracing::warn!("Unhandled property merge, other is None: {:?}", p);
            }
            (None, None) => {
                debug_assert!(false, "Unhandled property merge, both properties are None");
                tracing::warn!(message = "Unhandled property merge, both properties are None");
            }
        }
    }
}

impl Property {
    pub fn merge_property(&mut self, other: &Self) {
        match (self, other) {
            (
                ProtoIncrementOnlyUintProp(ref mut self_prop),
                ProtoIncrementOnlyUintProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (ProtoImmutableUintProp(ref mut self_prop), ProtoImmutableUintProp(ref other_prop)) => {
                self_prop.merge_property(other_prop)
            }
            (
                ProtoDecrementOnlyUintProp(ref mut self_prop),
                ProtoDecrementOnlyUintProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                ProtoDecrementOnlyIntProp(ref mut self_prop),
                ProtoDecrementOnlyIntProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (
                ProtoIncrementOnlyIntProp(ref mut self_prop),
                ProtoIncrementOnlyIntProp(ref other_prop),
            ) => self_prop.merge_property(other_prop),
            (ProtoImmutableIntProp(ref mut self_prop), ProtoImmutableIntProp(ref other_prop)) => {
                self_prop.merge_property(other_prop)
            }
            (ProtoImmutableStrProp(ref mut self_prop), ProtoImmutableStrProp(ref other_prop)) => {
                self_prop.merge_property(other_prop)
            }
            // technically we could improve type safety here by exhausting the combinations,
            // but I'm not going to type that all out right now
            (p, op) => {
                // Currently we don't guarantee that randomly generated nodes will have consistent
                // property types when they share a property name
                extra_assert!(debug_assert!(
                    false,
                    "Invalid property merge: {:?} {:?}",
                    p, op
                ));
                tracing::warn!("Invalid property merge: {:?} {:?}", p, op);
            }
        }
    }
}

impl From<Static> for IdStrategy {
    fn from(strategy: Static) -> IdStrategy {
        IdStrategy {
            strategy: Some(id_strategy::Strategy::Static(strategy)),
        }
    }
}

impl From<Session> for IdStrategy {
    fn from(strategy: Session) -> IdStrategy {
        IdStrategy {
            strategy: Some(id_strategy::Strategy::Session(strategy)),
        }
    }
}
impl std::string::ToString for Property {
    fn to_string(&self) -> String {
        match self {
            ProtoIncrementOnlyUintProp(increment_only_uint_prop) => {
                increment_only_uint_prop.to_string()
            }
            ProtoImmutableUintProp(immutable_uint_prop) => immutable_uint_prop.to_string(),
            ProtoDecrementOnlyUintProp(decrement_only_uint_prop) => {
                decrement_only_uint_prop.to_string()
            }
            ProtoDecrementOnlyIntProp(decrement_only_int_prop) => {
                decrement_only_int_prop.to_string()
            }
            ProtoIncrementOnlyIntProp(increment_only_int_prop) => {
                increment_only_int_prop.to_string()
            }
            ProtoImmutableIntProp(immutable_int_prop) => immutable_int_prop.to_string(),
            ProtoImmutableStrProp(immutable_str_prop) => immutable_str_prop.to_string(),
        }
    }
}

impl IncrementOnlyUintProp {
    pub fn as_inner(&self) -> u64 {
        self.prop
    }
    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="IncrementOnlyUintProp merge", self_prop=?self, other_prop=?other_prop);
        self.prop = std::cmp::max(self.prop, other_prop.prop);
    }
}
impl ImmutableUintProp {
    pub fn as_inner(&self) -> u64 {
        self.prop
    }
    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="ImmutableUintProp merge", self_prop=?self, other_prop=?other_prop);
        extra_assert! {debug_assert_eq!(*self, *other_prop)}
    }
}
impl DecrementOnlyUintProp {
    pub fn as_inner(&self) -> u64 {
        self.prop
    }
    pub fn merge_property(&mut self, other_prop: &Self) {
        self.prop = std::cmp::min(self.prop, other_prop.prop);
    }
}
impl DecrementOnlyIntProp {
    pub fn as_inner(&self) -> i64 {
        self.prop
    }
    pub fn merge_property(&mut self, other_prop: &Self) {
        self.prop = std::cmp::min(self.prop, other_prop.prop);
    }
}
impl IncrementOnlyIntProp {
    pub fn as_inner(&self) -> i64 {
        self.prop
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="IncrementOnlyIntProp merge", self_prop=?self, other_prop=?other_prop);
        self.prop = std::cmp::max(self.prop, other_prop.prop);
    }
}
impl ImmutableIntProp {
    pub fn as_inner(&self) -> i64 {
        self.prop
    }
    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="ImmutableIntProp merge", self_prop=?self, other_prop=?other_prop);
        extra_assert!(debug_assert_eq!(*self, *other_prop));
    }
}
impl ImmutableStrProp {
    pub fn as_inner(&self) -> &str {
        self.prop.as_str()
    }

    pub fn merge_property(&mut self, other_prop: &Self) {
        tracing::trace!(message="ImmutableStrProp merge", self_prop=?self, other_prop=?other_prop);
        extra_assert!(debug_assert_eq!(*self, *other_prop));
    }
}

impl std::string::ToString for IncrementOnlyUintProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl std::string::ToString for ImmutableUintProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl std::string::ToString for DecrementOnlyUintProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl std::string::ToString for DecrementOnlyIntProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl std::string::ToString for IncrementOnlyIntProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl std::string::ToString for ImmutableIntProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl std::string::ToString for ImmutableStrProp {
    fn to_string(&self) -> String {
        self.prop.to_string()
    }
}

impl std::string::ToString for NodeProperty {
    fn to_string(&self) -> String {
        match &self.property {
            Some(node_property::Property::IncrementOnlyUint(increment_only_uint_prop)) => {
                increment_only_uint_prop.to_string()
            }
            Some(node_property::Property::ImmutableUint(immutable_uint_prop)) => {
                immutable_uint_prop.to_string()
            }
            Some(node_property::Property::DecrementOnlyUint(decrement_only_uint_prop)) => {
                decrement_only_uint_prop.to_string()
            }
            Some(node_property::Property::DecrementOnlyInt(decrement_only_int_prop)) => {
                decrement_only_int_prop.to_string()
            }
            Some(node_property::Property::IncrementOnlyInt(increment_only_int_prop)) => {
                increment_only_int_prop.to_string()
            }
            Some(node_property::Property::ImmutableInt(immutable_int_prop)) => {
                immutable_int_prop.to_string()
            }
            Some(node_property::Property::ImmutableStr(immutable_str_prop)) => {
                immutable_str_prop.to_string()
            }
            None => panic!("Invalid property : {:?}", self),
        }
    }
}

impl NodeDescription {
    pub fn get_property(&self, name: impl AsRef<str>) -> Option<&NodeProperty> {
        self.properties.get(name.as_ref())
    }

    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<NodeProperty>) {
        self.properties.insert(name.into(), value.into());
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }
}

impl<T> From<T> for NodeProperty
where
    T: Into<Property>,
{
    fn from(t: T) -> Self {
        NodeProperty {
            property: Some(t.into()),
        }
    }
}

impl From<NodeDescription> for IdentifiedNode {
    fn from(n: NodeDescription) -> Self {
        IdentifiedNode {
            properties: n.properties,
            node_key: n.node_key,
            node_type: n.node_type,
        }
    }
}

impl MergedNode {
    pub fn from(n: IdentifiedNode, uid: u64) -> Self {
        Self {
            uid,
            properties: n.properties,
            node_key: n.node_key,
            node_type: n.node_type,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        extra_assert!(debug_assert_eq!(self.node_type, other.node_type));
        extra_assert!(debug_assert_eq!(self.node_key, other.node_key));
        for (prop_name, prop_value) in other.properties.iter() {
            match self.properties.get_mut(prop_name) {
                Some(self_prop) => self_prop.merge(prop_value),
                None => {
                    self.properties
                        .insert(prop_name.clone(), prop_value.clone());
                }
            }
        }
    }

    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

impl IdentifiedNode {
    pub fn into(self, uid: u64) -> MergedNode {
        MergedNode {
            uid,
            properties: self.properties,
            node_key: self.node_key,
            node_type: self.node_type,
        }
    }
    pub fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    pub fn clone_node_key(&self) -> String {
        self.node_key.clone()
    }
}

// These are helper types that let us easily distinguish between variants of the Property type

impl_from_for_unit!(
    ImmutableUintProp,
    prop,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);
impl_from_for_unit!(
    IncrementOnlyUintProp,
    prop,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);
impl_from_for_unit!(
    DecrementOnlyUintProp,
    prop,
    u64,
    u32,
    u16,
    u8,
    &u64,
    &u32,
    &u16,
    &u8
);
impl_from_for_unit!(
    ImmutableIntProp,
    prop,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);
impl_from_for_unit!(
    IncrementOnlyIntProp,
    prop,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);
impl_from_for_unit!(
    DecrementOnlyIntProp,
    prop,
    i64,
    i32,
    i16,
    i8,
    &i64,
    &i32,
    &i16,
    &i8
);
impl_from_for_unit!(ImmutableStrProp, prop, String, &String, &str);

impl From<ImmutableUintProp> for Property {
    fn from(p: ImmutableUintProp) -> Self {
        Self::ImmutableUint(p)
    }
}
impl From<IncrementOnlyUintProp> for Property {
    fn from(p: IncrementOnlyUintProp) -> Self {
        Self::IncrementOnlyUint(p)
    }
}
impl From<DecrementOnlyUintProp> for Property {
    fn from(p: DecrementOnlyUintProp) -> Self {
        Self::DecrementOnlyUint(p)
    }
}
impl From<ImmutableIntProp> for Property {
    fn from(p: ImmutableIntProp) -> Self {
        Self::ImmutableInt(p)
    }
}
impl From<IncrementOnlyIntProp> for Property {
    fn from(p: IncrementOnlyIntProp) -> Self {
        Self::IncrementOnlyInt(p)
    }
}
impl From<DecrementOnlyIntProp> for Property {
    fn from(p: DecrementOnlyIntProp) -> Self {
        Self::DecrementOnlyInt(p)
    }
}
impl From<ImmutableStrProp> for Property {
    fn from(p: ImmutableStrProp) -> Self {
        Self::ImmutableStr(p)
    }
}

impl NodeProperty {
    pub fn as_increment_only_uint(&self) -> Option<IncrementOnlyUintProp> {
        match self.property {
            Some(ProtoIncrementOnlyUintProp(ref prop)) => Some(*prop),
            _ => None,
        }
    }

    pub fn as_immutable_uint(&self) -> Option<ImmutableUintProp> {
        match self.property {
            Some(ProtoImmutableUintProp(ref prop)) => Some(*prop),
            _ => None,
        }
    }

    pub fn as_decrement_only_uint(&self) -> Option<DecrementOnlyUintProp> {
        match self.property {
            Some(ProtoDecrementOnlyUintProp(ref prop)) => Some(*prop),
            _ => None,
        }
    }

    pub fn as_decrement_only_int(&self) -> Option<DecrementOnlyIntProp> {
        match self.property {
            Some(ProtoDecrementOnlyIntProp(ref prop)) => Some(*prop),
            _ => None,
        }
    }

    pub fn as_increment_only_int(&self) -> Option<IncrementOnlyIntProp> {
        match self.property {
            Some(ProtoIncrementOnlyIntProp(ref prop)) => Some(*prop),
            _ => None,
        }
    }

    pub fn as_immutable_int(&self) -> Option<ImmutableIntProp> {
        match self.property {
            Some(ProtoImmutableIntProp(ref prop)) => Some(*prop),
            _ => None,
        }
    }

    pub fn as_immutable_str(&self) -> Option<&ImmutableStrProp> {
        match self.property {
            Some(ProtoImmutableStrProp(ref prop)) => Some(prop),
            _ => None,
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::{
        collections::HashMap,
        hash::Hasher,
    };

    #[cfg(not(feature = "fuzzing"))]
    use quickcheck::{
        Arbitrary,
        Gen,
    };
    use quickcheck_macros::quickcheck;

    use super::*;

    impl Arbitrary for IncrementOnlyIntProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: i64::arbitrary(g),
            }
        }
    }
    impl Arbitrary for DecrementOnlyIntProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: i64::arbitrary(g),
            }
        }
    }
    impl Arbitrary for ImmutableIntProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: i64::arbitrary(g),
            }
        }
    }
    impl Arbitrary for IncrementOnlyUintProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: u64::arbitrary(g),
            }
        }
    }
    impl Arbitrary for DecrementOnlyUintProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: u64::arbitrary(g),
            }
        }
    }
    impl Arbitrary for ImmutableUintProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: u64::arbitrary(g),
            }
        }
    }
    impl Arbitrary for ImmutableStrProp {
        fn arbitrary(g: &mut Gen) -> Self {
            Self {
                prop: String::arbitrary(g),
            }
        }
    }

    impl Arbitrary for Property {
        fn arbitrary(g: &mut Gen) -> Self {
            let props = &[
                Property::IncrementOnlyInt(IncrementOnlyIntProp::arbitrary(g)),
                Property::DecrementOnlyInt(DecrementOnlyIntProp::arbitrary(g)),
                Property::ImmutableInt(ImmutableIntProp::arbitrary(g)),
                Property::IncrementOnlyUint(IncrementOnlyUintProp::arbitrary(g)),
                Property::DecrementOnlyUint(DecrementOnlyUintProp::arbitrary(g)),
                Property::ImmutableUint(ImmutableUintProp::arbitrary(g)),
                Property::ImmutableStr(ImmutableStrProp::arbitrary(g)),
            ];
            g.choose(props).unwrap().clone()
        }
    }

    impl Arbitrary for NodeProperty {
        fn arbitrary(g: &mut Gen) -> Self {
            NodeProperty {
                property: Some(Property::arbitrary(g)),
            }
        }
    }

    fn hash(bytes: &[impl AsRef<[u8]>]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for _bytes in bytes {
            hasher.write(_bytes.as_ref());
        }
        hasher.finish() as u64
    }

    fn choice<T: Clone>(bytes: impl AsRef<[u8]>, choices: &[T]) -> T {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(bytes.as_ref());
        let choice_index = (hasher.finish() as usize) % choices.len();
        choices[choice_index].clone()
    }

    fn choose_property(node_key: &str, property_name: &str, g: &mut Gen) -> NodeProperty {
        let s = format!("{}{}", node_key, property_name);

        let props = &[
            Property::IncrementOnlyInt(IncrementOnlyIntProp::arbitrary(g)),
            Property::DecrementOnlyInt(DecrementOnlyIntProp::arbitrary(g)),
            Property::IncrementOnlyUint(IncrementOnlyUintProp::arbitrary(g)),
            Property::DecrementOnlyUint(DecrementOnlyUintProp::arbitrary(g)),
            Property::ImmutableInt(ImmutableIntProp::from(
                hash(&[node_key, property_name]) as i64
            )),
            Property::ImmutableUint(ImmutableUintProp::from(hash(&[node_key, property_name]))),
            Property::ImmutableStr(ImmutableStrProp::from(s)),
        ];
        let p: Property = choice(node_key, props);
        p.into()
    }

    impl Arbitrary for IdentifiedNode {
        fn arbitrary(g: &mut Gen) -> Self {
            let node_keys = &[
                "c413e25e-9c50-4faf-8e61-f8bfb0e0d18e".to_string(),
                "0d5c9261-2b6e-4094-8de3-b349cb0aa310".to_string(),
                "ed1f73df-f38d-43c0-87b0-5aff06e1f68b".to_string(),
                "6328e956-117e-4f7f-8a5b-c56be1111f43".to_string(),
            ];
            let node_key = g.choose(node_keys).unwrap().clone();

            let node_types = &["Process", "File", "IpAddress"];
            let node_type = choice(&node_key, node_types);
            let mut properties = HashMap::new();
            let property_names: Vec<String> = Vec::arbitrary(g);
            for property_name in property_names {
                let property = choose_property(&node_key, &property_name, g);
                properties.insert(property_name.to_owned(), property);
            }
            IdentifiedNode {
                node_key: node_key.to_owned(),
                node_type: node_type.to_owned(),
                properties,
            }
        }
    }

    fn init_test_env() {
        let subscriber = ::tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(::tracing_subscriber::EnvFilter::from_default_env())
            .finish();
        let _ = ::tracing::subscriber::set_global_default(subscriber);
    }

    // These tests mostly target ensuring that our merge functions are commutative and idempotent
    // That said - immutable data is *not* commutative. Therefor, assertions around commutativity
    // are disabled for for tests on immutable data via the "extra_assertions" feature

    #[cfg(not(feature = "extra_assertions"))]
    #[quickcheck]
    fn test_merge_str(x: ImmutableStrProp, y: ImmutableStrProp) {
        init_test_env();
        let original = x;
        let mut x = original.clone();
        x.merge_property(&y);
        assert_eq!(original, x);
    }

    #[cfg(not(feature = "extra_assertions"))]
    #[quickcheck]
    fn test_merge_immutable_int(mut x: ImmutableIntProp, y: ImmutableIntProp) {
        init_test_env();
        let original = x.clone();
        x.merge_property(&y);
        assert_eq!(x, original);
    }

    #[cfg(not(feature = "extra_assertions"))]
    #[quickcheck]
    fn test_merge_immutable_uint(mut x: ImmutableUintProp, y: ImmutableUintProp) {
        init_test_env();
        let original = x.clone();
        x.merge_property(&y);
        assert_eq!(x, original);
    }

    #[quickcheck]
    fn test_merge_uint_max(mut x: IncrementOnlyUintProp, y: IncrementOnlyUintProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x, std::cmp::max(x, y));
    }

    #[quickcheck]
    fn test_merge_int_max(mut x: IncrementOnlyIntProp, y: IncrementOnlyIntProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x, std::cmp::max(x, y));
    }

    #[quickcheck]
    fn test_merge_uint_min(mut x: DecrementOnlyUintProp, y: DecrementOnlyUintProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x, std::cmp::min(x, y));
    }

    #[quickcheck]
    fn test_merge_int_min(mut x: DecrementOnlyIntProp, y: DecrementOnlyIntProp) {
        init_test_env();
        x.merge_property(&y);
        assert_eq!(x, std::cmp::min(x, y));
    }

    #[quickcheck]
    fn test_merge_incr_uint_commutative(mut properties: Vec<IncrementOnlyUintProp>) {
        init_test_env();
        if properties.is_empty() {
            return;
        }
        properties.sort_unstable();
        let max_value = properties.iter().max().unwrap().to_owned();
        let mut first_x = properties[0].clone();
        for property in properties.iter() {
            first_x.merge_property(property)
        }

        let properties: Vec<_> = properties.into_iter().rev().collect();
        let mut first_y = properties[0].clone();
        for property in properties.iter() {
            first_y.merge_property(property)
        }
        assert_eq!(first_x, first_y);
        assert_eq!(first_x, max_value);
    }

    #[quickcheck]
    fn test_merge_identified_node(mut node_0: IdentifiedNode, node_1: IdentifiedNode) {
        if node_0.node_key != node_1.node_key {
            return;
        }
        // let original = node_0.clone();
        node_0.merge(&node_1);

        // for (_o_pred_name, o_pred_val) in original.iter() {
        //     let mut copy = o_pred_val.clone();
        // }
    }
}

pub(crate) mod graplinc {
    pub(crate) mod grapl {
        pub(crate) mod api {
            pub(crate) mod graph {
                pub(crate) mod v1beta1 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/graplinc.grapl.api.graph.v1beta1.rs"
                    ));
                }
            }

            pub(crate) mod services {
                pub(crate) mod v1beta1 {
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/graplinc.grapl.api.services.v1beta1.rs"
                    ));
                }
            }
        }
    }
}

pub mod graph_descriptions;
pub use graph_descriptions::node_property;

pub mod services;

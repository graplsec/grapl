use rust_proto::graph_descriptions::*;
use serde::{
    Deserialize,
    Serialize,
};

mod grapl_pack;

#[derive(Serialize, Deserialize, Clone, Hash)]
#[serde(tag = "name")]
pub enum OSQueryEvent {
    #[serde(rename = "pack_grapl_processes")]
    Process(grapl_pack::processes::ProcessEvent),
    #[serde(rename = "pack_grapl_process-files")]
    ProcessFileAction(grapl_pack::process_files::ProcessFileInteractionEvent),
    #[serde(rename = "pack_grapl_files")]
    File(grapl_pack::files::FileEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub(self) enum OSQueryAction {
    Added,
    Removed,
    Other(String),
}

impl From<OSQueryEvent> for GraphDescription {
    fn from(event: OSQueryEvent) -> Self {
        match event {
            OSQueryEvent::File(event) => event.into(),
            OSQueryEvent::Process(event) => event.into(),
            OSQueryEvent::ProcessFileAction(event) => event.into(),
        }
    }
}

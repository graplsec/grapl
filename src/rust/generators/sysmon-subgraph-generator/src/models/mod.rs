use chrono::{
    DateTime,
    NaiveDateTime,
    Utc,
};
use rust_proto::graph_descriptions::*;
use sysmon::Event;

use crate::generator::SysmonGeneratorError;

mod file;
mod network;
mod process;

/// Because this crate doesn't own sysmon::Event nor rust_proto::graph_descriptions::GraphDescription
/// we need to create a new Trait to add a function to graph for Event.
///
/// Mimics TryFrom
pub(crate) trait SysmonTryFrom<T>: Sized {
    type Error;

    fn try_from(instance: T) -> Result<Self, Self::Error>;
}

fn get_event_type(event: Event) -> String {
    match event {
        Event::ProcessCreate(event) => event.system.event_id.event_id.to_string(),
        Event::FileCreate(event) => event.system.event_id.event_id.to_string(),
        Event::InboundNetwork(event) => event.system.event_id.event_id.to_string(),
        Event::OutboundNetwork(event) => event.system.event_id.event_id.to_string(),
    }
}

impl SysmonTryFrom<Event> for GraphDescription {
    type Error = SysmonGeneratorError;

    #[tracing::instrument]
    fn try_from(instance: Event) -> Result<Self, Self::Error> {
        match instance {
            Event::ProcessCreate(event) => {
                tracing::info!(event = "ProcessCreate");

                let result = process::generate_process_create_subgraph(&event);

                if let Err(e) = &result {
                    tracing::warn!(message="Failed to process process start event.", error=?e);
                }

                result
            }
            Event::FileCreate(event) => {
                tracing::info!(event = "FileCreate");

                let result = file::generate_file_create_subgraph(&event);

                if let Err(e) = &result {
                    tracing::warn!(message="Failed to process file create event", error=?e);
                }

                result
            }
            // Event::InboundNetwork(event) => {
            //     info!("InboundNetwork");
            //
            //     let result = network::generate_inbound_connection_subgraph(&event);
            //
            //     if let Err(e) = &result {
            //         warn!("Failed to process inbound network event: {}", e);
            //     }
            //
            //     result
            // }
            Event::OutboundNetwork(event) => {
                tracing::info!(event = "OutboundNetwork");

                let result = network::generate_outbound_connection_subgraph(&event);

                if let Err(e) = &result {
                    tracing::warn!(message="Failed to process outbound network event.", error=?e);
                }

                result
            }
            unsupported_event => {
                let message = format!("Unsupported event_type: {:?}", unsupported_event);

                tracing::warn!(message =% message);

                Err(SysmonGeneratorError::UnsupportedEventType(get_event_type(
                    unsupported_event,
                )))
            }
        }
    }
}

/// Returns the provided file path with the Windows Zone Identifier removed if present.
///
/// When files are downloaded via a browser (e.g. Internet Explorer), an alternative data stream (ADS) may be created
/// to store additional meta-data about the file. These ADS files are suffixed with `:Zone.Identifier`.
///
/// The purpose of these ADS files is to provide context to users and Windows Policy controls to keep users safe.
///
/// For example, Microsoft Word may open potentially unsafe `.docx` files marked with `:Zone.Identifier`
/// in Protected View if the URL security zone is not well trusted.
///
/// ["What are Zone Identifier Files?"](https://stackoverflow.com/questions/4496697/what-are-zone-identifier-files)
fn strip_file_zone_identifier(path: &str) -> &str {
    if path.ends_with(":Zone.Identifier") {
        &path[0..path.len() - ":Zone.Identifier".len()]
    } else {
        path
    }
}

/// Gets the name of the process given a path to the executable.
fn get_image_name(image_path: &str) -> Option<String> {
    image_path
        .split('\\')
        .last()
        .map(|name| name.replace("- ", "").replace('\\', ""))
}

/// Converts a Sysmon UTC string to UNIX Epoch time
///
/// If the provided string is not parseable as a UTC timestamp, an error is returned.
pub fn utc_to_epoch(utc: &str) -> Result<u64, SysmonGeneratorError> {
    let dt = NaiveDateTime::parse_from_str(utc, "%Y-%m-%d %H:%M:%S%.3f")?;

    let dt: DateTime<Utc> = DateTime::from_utc(dt, Utc);
    let ts = dt.timestamp_millis();

    if ts < 0 {
        return Err(SysmonGeneratorError::NegativeEventTime(ts));
    }

    if ts < 1_000_000_000_000 {
        Ok(ts as u64 * 1000)
    } else {
        Ok(ts as u64)
    }
}

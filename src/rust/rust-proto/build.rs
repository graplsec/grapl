fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    println!("cargo:rerun-if-changed={}", "Cargo.toml");
    println!("cargo:rerun-if-changed={}", "Cargo.lock");
    println!("cargo:rerun-if-changed={}", "build.rs");

    change_on_dir("../../proto/")?;
    change_on_dir("src/")?;
    config.type_attribute(
        ".",
        "#[derive(Eq, serde_derive::Serialize, serde_derive::Deserialize)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IncrementOnlyIntProp",
        "#[derive(Copy, Ord, PartialOrd)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.DecrementOnlyIntProp",
        "#[derive(Copy, Ord, PartialOrd)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ImmutableIntProp",
        "#[derive(Copy, Ord, PartialOrd)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IncrementOnlyUintProp",
        "#[derive(Copy, Ord, PartialOrd)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.DecrementOnlyUintProp",
        "#[derive(Copy, Ord, PartialOrd)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ImmutableUintProp",
        "#[derive(Copy, Ord, PartialOrd)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.MergedEdge",
        "#[derive(Ord, PartialOrd)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Edge",
        "#[derive(Ord, PartialOrd)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpAddress",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpPort",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.NetworkConnection",
        "#[derive(Builder)]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpConnection",
        "#[derive(Builder)]",
    );

    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpAddress",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpPort",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.NetworkConnection",
        "#[builder(setter(into))]",
    );
    config.type_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpConnection",
        "#[builder(setter(into))]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.created_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.deleted_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.last_seen_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_name",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_path",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_extension",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_mime_type",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_size",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_version",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_description",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_product",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_company",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_directory",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_inode",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.file_hard_links",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.md5_hash",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.sha1_hash",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.sha256_hash",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.asset_id",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.hostname",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.File.host_ip",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.process_id",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.process_guid",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.created_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.terminated_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.last_seen_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.process_name",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.process_command_line",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.process_integrity_level",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.operating_system",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.asset_id",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.hostname",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Process.host_ip",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.asset_id",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.hostname",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.host_ip",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.created_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.terminated_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessInboundConnection.last_seen_timestamp",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.asset_id",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.hostname",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.host_ip",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.created_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.terminated_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.ProcessOutboundConnection.last_seen_timestamp",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.asset_id",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.ip_address",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.hostname",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.mac_address",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.first_seen_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.Asset.last_seen_timestamp",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpAddress.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpAddress.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpAddress.first_seen_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpAddress.last_seen_timestamp",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpPort.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpPort.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpPort.first_seen_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpPort.last_seen_timestamp",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.NetworkConnection.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.NetworkConnection.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.NetworkConnection.created_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.NetworkConnection.terminated_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.NetworkConnection.last_seen_timestamp",
        "#[builder(default)]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpConnection.node_key",
        "#[builder(field(private))]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpConnection.node_key",
        "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]",
    );

    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpConnection.created_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpConnection.terminated_timestamp",
        "#[builder(default)]",
    );
    config.field_attribute(
        ".graplinc.grapl.api.graph.v1beta1.IpConnection.last_seen_timestamp",
        "#[builder(default)]",
    );

    config
        .compile_protos(
            &[
                "../../proto/graplinc/grapl/api/graph/v1beta1/types.proto",
                "../../proto/graplinc/grapl/api/services/v1beta1/types.proto",
            ],
            &["../../proto/"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
    Ok(())
}

fn change_on_dir(root_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    for entry in std::fs::read_dir(current_dir.join(root_dir))? {
        let entry = entry?;
        if !entry.metadata()?.is_file() {
            continue;
        }
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.display());
    }
    Ok(())
}

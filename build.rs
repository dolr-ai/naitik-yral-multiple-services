use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "contracts/projects/warehouse_events/warehouse_events.proto";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_client(false)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("warehouse_events_descriptor.bin"))
        .out_dir(out_dir)
        .compile_protos(&[proto_file], &["proto"])?;

    Ok(())
}

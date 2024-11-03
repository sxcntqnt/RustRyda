use tonic::transport::Server;
use std::fs;
use std::path::Path;
use std::process::Command;

use server::StoreInventory;
use store::inventory_server::InventoryServer;

pub mod server;
pub mod store;

mod store_proto {
    include!("store.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("store_descriptor");
}

fn run_all_programs(src_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = Path::new(src_dir);

    // Check if the specified directory exists and is a directory
    if !src_path.exists() || !src_path.is_dir() {
        eprintln!("Directory {} does not exist or is not a directory.", src_dir);
        return Ok(());
    }

    let mut children = Vec::new();

    // Iterate over all files in the specified directory
    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let path = entry.path();

        // Only process files (skip directories)
        if path.is_file() {
            let command = path.to_str().expect("Invalid path");

            // Spawn each command in a separate shell instance
            let child = Command::new("cmd")
                .args(&["/C", "start", "cmd", "/K", command])
                .spawn()
                .expect("Failed to execute command");

            children.push(child);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run all programs in the specified directory
    if let Err(e) = run_all_programs("../src") {
        eprintln!("Error running programs: {}", e);
    }

    // Set up server address and inventory service
    let addr = "127.0.0.1:9001".parse()?;
    let inventory = StoreInventory::default();

    // Configure and register reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(store_proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    // Start the server with inventory and reflection services
    Server::builder()
        .add_service(InventoryServer::new(inventory))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}

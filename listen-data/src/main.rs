fn main() {
    println!("Listen Data Service");
    println!("-------------------");
    println!("\nAvailable commands:");
    println!("\n1. indexer");
    println!("   Geyser-based indexer for Raydium data");
    println!("   Usage: cargo run --bin indexer");
    println!("\n2. rpc-crawler");
    println!("   RPC-based crawler for Raydium data");
    println!("   Usage: cargo run --bin rpc-crawler [COMMAND]");
    println!("   Commands:");
    println!("     - raydium-accounts-rpc");
    println!("     - raydium-instrutions-rpc");
    println!("\nFor more details, run any command with --help");
}

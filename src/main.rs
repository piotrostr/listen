use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    signature: String,
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    url: String,

    #[arg(short, long, default_value = "wss://api.mainnet-beta.solana.com")]
    ws_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let rpc_client = listen::get_client(&args.url)?;
    let listener = listen::Listener {
        url: args.url,
        ws_url: args.ws_url,
        rpc_client,
    };
    if args.signature != "" {
        let tx = listener.get_tx(&args.signature)?;
        let mint = listener.parse_mint(&tx)?;
        println!("Mint: {:?}", mint);
        let pricing = listener.get_pricing(&mint).await?;
        println!("Pricing: {:?}", pricing);
        return Ok(());
    }
    Ok(listener.logs_subscribe()?)
}

use clap::Parser;
use listen::util;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    signature: String,

    #[arg(short, long)]
    listen: bool,

    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    url: String,

    #[arg(short, long, default_value = "wss://api.mainnet-beta.solana.com")]
    ws_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 30th April, let's see how well this ages lol
    let sol_price = 135.;
    let args = Args::parse();
    let listener = listen::Listener::new(args.ws_url);
    let provider = listen::Provider::new(args.url);
    if args.signature != "" {
        let tx = provider.get_tx(&args.signature)?;
        println!("Transaction: {:}", serde_json::to_string_pretty(&tx)?);
        let mint = listener.parse_mint(&tx)?;
        println!("Mint: {:?}", mint);
        let pricing = provider.get_pricing(&mint).await?;
        println!("Pricing: {:?}", pricing);

        let swap = listener.parse_token_transfers(&tx)?;
        println!("Swap: {}", serde_json::to_string_pretty(&swap)?);

        let sol_notional =
            listen::util::lamports_to_sol(swap.quote_amount as u64);

        let usd_notional = sol_notional * sol_price;

        println!("{} ({} USD)", sol_notional, usd_notional);

        return Ok(());
    }

    if args.listen {
        listener.logs_subscribe()?
    }

    Ok(())
}

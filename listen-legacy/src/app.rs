use clap::Parser;

#[derive(Parser, Debug)]
pub struct App {
    #[clap(flatten)]
    pub args: Args,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[arg(short, long, default_value = "https://api.mainnet-beta.solana.com")]
    pub url: String,

    #[arg(short, long, default_value = "wss://api.mainnet-beta.solana.com")]
    pub ws_url: String,

    #[arg(short, long)]
    pub keypair_path: Option<String>,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub tokio_console: Option<bool>,
}

#[derive(Debug, Parser)]
pub enum Command {
    ListenService {
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    ArcAgent {},
    BundleStatus {
        #[arg(long)]
        bundle: String,
    },
    DownloadRaydiumJson {
        #[arg(long, action = clap::ArgAction::SetTrue)]
        update: Option<bool>,
    },
    SweepRaydium {
        #[arg(long)]
        wallet_path: String,
    },
    CloseTokenAccounts {
        #[arg(long)]
        wallet_path: String,
    },
    PumpService {},
    GrabMetadata {
        #[arg(long)]
        mint: String,
    },
    SellPump {
        #[arg(long)]
        mint: String,
    },
    BumpPump {
        #[arg(long)]
        mint: String,
    },
    SweepPump {
        #[arg(long)]
        wallet_path: String,
    },
    SnipePump {
        #[arg(long, action = clap::ArgAction::SetTrue)]
        only_listen: Option<bool>,
    },
    BuyPumpToken {
        #[arg(long)]
        mint: String,
    },
    GenerateCustomAddress {
        #[arg(long)]
        prefixes: Vec<String>,
    },
    Ata {
        #[arg(long)]
        mint: String,
    },
    SplStream {
        #[arg(long)]
        ata: String,
    },
    MonitorMempool {},
    SellerService {},
    CheckerService {},
    Checks {
        #[arg(long)]
        signature: String,
    },
    Blockhash {},
    ListenForSolPooled {
        #[arg(long)]
        amm_pool: String,
    },
    BuyerService {},
    TrackPosition {
        #[arg(long)]
        amm_pool: String,

        #[arg(long)]
        owner: String,
    },
    TopHolders {
        #[arg(long)]
        mint: String,
    },
    MonitorLeaders {},
    MonitorSlots {},
    Price {
        #[arg(long)]
        amm_pool: String,
    },
    BenchRPC {
        #[arg(long)]
        rpc_url: String,
    },
    PriorityFee {},
    Tx {
        #[arg(short, long)]
        signature: String,
    },
    Listen {
        #[arg(long, default_value_t = 10)]
        worker_count: i32,

        #[arg(long, default_value_t = 10)]
        buffer_size: i32,
    },
    ListenForBurn {
        #[arg(long)]
        amm_pool: String,
    },
    ListenerService {
        #[arg(long, action = clap::ArgAction::SetTrue)]
        webhook: Option<bool>,
    },
    Snipe {},
    Wallet {},
    ParsePool {
        #[arg(long)]
        signature: String,
    },
    Swap {
        #[arg(long)]
        input_mint: String,
        #[arg(long)]
        output_mint: String,
        #[arg(long)]
        amount: Option<i64>,
        #[arg(long)]
        slippage: Option<u16>,
        #[arg(long)]
        dex: Option<String>,
        #[arg(long)]
        amm_pool_id: Option<String>,

        #[clap(short, long, action = clap::ArgAction::SetTrue)]
        yes: Option<bool>,
    },
}

import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { gruvboxDark } from "react-syntax-highlighter/dist/esm/styles/prism";

const App = () => {
  return (
    <div className="relative min-h-screen bg-black text-white">
      <div className="fixed inset-0 w-screen h-screen bg-[url('/bg.webp')] bg-cover bg-center opacity-10" />

      <div className="relative z-10 max-w-6xl mx-auto px-4 py-20">
        {/* Hero */}
        <div className="text-center mb-20">
          <img
            src="/listen.svg"
            alt="listen"
            className="w-32 h-32 mx-auto mb-12"
          />
          <h1 className="text-6xl font-bold mb-6">listen-rs</h1>
          <p className="text-xl text-gray-300">
            blazingly fast actions for AI agents in Rust
          </p>
          <div className="mt-5 flex flex-row justify-center items-center space-x-4">
            <a href="https://github.com/piotrostr/listen">
              <img
                src="https://img.shields.io/github/stars/piotrostr/listen?style=social"
                alt="GitHub Stars"
              />
            </a>
            <a href="https://github.com/piotrostr/listen/blob/main/LICENSE">
              <img
                src="https://img.shields.io/github/license/piotrostr/listen"
                alt="License"
              />
            </a>
          </div>
        </div>

        {/* Features Grid */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
          <div className="p-6 rounded-xl bg-gray-800/50 backdrop-blur transition-transform hover:scale-[1.02]">
            <h3 className="text-xl font-bold mb-3">Token Analysis & Checks</h3>
            <pre className="text-sm text-gray-300 font-mono whitespace-pre-line">
              • Comprehensive holder analysis
              <br />
              • Ownership concentration checks
              <br />
              • Mint authority verification
              <br />• Program authority analysis
              <br />• Metadata validation
            </pre>
          </div>

          <div className="p-6 rounded-xl bg-gray-800/50 backdrop-blur transition-transform hover:scale-[1.02]">
            <h3 className="text-xl font-bold mb-3">Real-time Monitoring</h3>
            <pre className="text-sm text-gray-300 font-mono whitespace-pre-line">
              • Transaction tracking
              <br />
              • Configurable worker threads
              <br />
              • Adjustable buffer sizes
              <br />
              • Prometheus metrics integration
              <br />• WebSocket subscription handling
            </pre>
          </div>

          <div className="p-6 rounded-xl bg-gray-800/50 backdrop-blur transition-transform hover:scale-[1.02]">
            <h3 className="text-xl font-bold mb-3">Advanced Swapping</h3>
            <pre className="text-sm text-gray-300 font-mono whitespace-pre-line">
              • Multi-platform swap execution
              <br />
              • Support for Pump.fun
              <br />
              • Jupiter V6 API integration
              <br />
              • Direct Raydium interaction
              <br />• Custom slippage settings
            </pre>
          </div>

          <div className="p-6 rounded-xl bg-gray-800/50 backdrop-blur transition-transform hover:scale-[1.02]">
            <h3 className="text-xl font-bold mb-3">Price Tracking</h3>
            <pre className="text-sm text-gray-300 font-mono whitespace-pre-line">
              • Real-time price monitoring
              <br />
              • PubSub subscription system
              <br />
              • Webhook integration
              <br />
              • Price alert configuration
              <br />• Historical data tracking
            </pre>
          </div>

          <div className="p-6 rounded-xl bg-gray-800/50 backdrop-blur transition-transform hover:scale-[1.02]">
            <h3 className="text-xl font-bold mb-3">Token Management</h3>
            <pre className="text-sm text-gray-300 font-mono whitespace-pre-line">
              • Custom wallet generation
              <br />
              • Batch token account closing
              <br />
              • Token sweeping functionality
              <br />
              • ATA Sweeps
              <br />• Balance consolidation
            </pre>
          </div>

          <div className="p-6 rounded-xl bg-gray-800/50 backdrop-blur transition-transform hover:scale-[1.02]">
            <h3 className="text-xl font-bold mb-3">Performance Tools</h3>
            <pre className="text-sm text-gray-300 font-mono whitespace-pre-line">
              • Transaction profiling
              <br />
              • RPC benchmarking
              <br />
              • Real-time priority fee
              <br />
              • Memory usage tracking
              <br />• Latency monitoring
            </pre>
          </div>
        </div>

        <div className="mt-20 bg-gray-800/50 rounded-xl p-8">
          <h2 className="text-2xl font-bold mb-4">Quick Start</h2>
          <div className="mb-4">
            While $ARC provides the base infrastructure for interacting with AI
            Agents, it lacks the logic for performing on-chain actions. This
            library provides this functionality in asynchronous Rust, allowing
            for high-performance and reliable execution.
            <br />
            <br />
            With listen-rs, you can easily pipe blockchain interactions into
            your agent
          </div>
          <h3 className="text-xl font-bold mb-4">Installation</h3>
          <pre className="bg-black/50 p-4 rounded-lg overflow-x-auto text-sm mb-3">
            <SyntaxHighlighter
              language="bash"
              style={{
                ...gruvboxDark,
                'pre[class*="language-"]': {
                  ...gruvboxDark['pre[class*="language-"'],
                  background: "transparent",
                },
              }}
            >
              {`sudo apt install protoc build-essential pkg-config libssl-dev
git clone https://github.com/piotrostr/listen && cd listen
cp .env.example .env  # swap the example values with your RPCs
cargo build --release
`}
            </SyntaxHighlighter>
          </pre>
          <br />
          <h3 className="text-xl font-bold mb-4">
            Example (top holders check)
          </h3>
          <pre className="bg-black/50 p-4 rounded-lg overflow-x-auto text-sm mb-3">
            <SyntaxHighlighter
              language="rust"
              style={{
                ...gruvboxDark,
                'pre[class*="language-"]': {
                  ...gruvboxDark['pre[class*="language-"]'],
                  background: "transparent",
                },
              }}
            >
              {`// import the check_top_holders from \`listen\` lib
use listen::buyer::check_top_holders;
// import the rig framework Tool trait
use rig::{completion::ToolDefinition, tool::Tool};

#[derive(Deserialize, Serialize)]
pub struct TopHolders;
impl Tool for TopHolders {
	async fn call(
		&self,
		args: Self::Args,
	) -> Result<Self::Output, Self::Error> {
		let mint = Pubkey::from_str(&args.mint)
			.map_err(|e| TopHoldersError::InvalidMint(e.to_string()))?;

		// Create a channel
		let (tx, mut rx) = mpsc::channel(1);

		// Spawn a task to handle the RPC calls
		tokio::spawn(async move {
			let provider = Provider::new(env("RPC_URL"));
			let result = check_top_holders(&mint, &provider, true).await;
			let _ = tx.send(result).await;
		});

		// Wait for the result
		let result = rx
			.recv()
			.await
			.ok_or_else(|| {
					TopHoldersError::CheckFailed("Channel closed".to_string())
			})?
			.map_err(|e| TopHoldersError::CheckFailed(e.to_string()))?;

		let (percentage, is_concentrated, details) = result;

		Ok(TopHoldersOutput {
			percentage,
			is_concentrated,
			details,
		})
	}
}
`}
            </SyntaxHighlighter>
          </pre>
          <div className="mb-4 text-sm ">
            full code:{" "}
            <a
              href="https://github.com/piotrostr/listen/blob/main/src/agent.rs"
              className="text-blue-400 hover:underline"
            >
              src/agent.rs
            </a>
          </div>
          <br />
          <h3 className="text-xl font-bold mb-4">All available actions</h3>
          <pre className="bg-black/50 p-4 rounded-lg overflow-x-auto text-sm">
            <code className="text-gray-300">
              {`$ listen
Usage: listen [OPTIONS] <COMMAND>

Commands:
  close-token-accounts
  pump-service
  grab-metadata
  sell-pump
  bump-pump
  sweep-pump
  snipe-pump
  buy-pump-token
  generate-custom-address
  ata
  spl-stream
  monitor-mempool
  seller-service
  checker-service
  checks
  blockhash
  listen-for-sol-pooled
  buyer-service
  track-position
  top-holders
  monitor-leaders
  monitor-slots
  price
  bench-rpc
  priority-fee
  tx
  listen
  listen-for-burn
  listener-service
  snipe
  wallet
  parse-pool
  swap
  help                 Print this message or the help of the given subcommand(s)

Options:
  -u, --url <URL>                 [default: https://api.mainnet-beta.solana.com]
  -w, --ws-url <WS_URL>           [default: wss://api.mainnet-beta.solana.com]
  -k, --keypair-path <KEYPAIR_PATH>
      --tokio-console
  -h, --help                      Print help
  -V, --version                   Print version

`}
            </code>
          </pre>
        </div>
      </div>
    </div>
  );
};

export default App;

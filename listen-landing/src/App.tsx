import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Header } from "./Header";
import { Background } from "./Background";

const App = () => {
  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />
      <div className="relative z-10 max-w-6xl mx-auto px-4 py-20">
        {/* Hero */}
        <div className="text-center mb-20 mt-10">
          <img
            src="/listen-more.png"
            alt="listen"
            className="w-48 h-48 mx-auto mb-12 rounded shadow-lg"
          />
          <h1 className="text-6xl font-bold mb-6">listen</h1>
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
          <div className="mt-5 flex justify-center text-lg [&>pre]:rounded-lg">
            <SyntaxHighlighter
              language="bash"
              style={{
                ...vscDarkPlus,
                'code[class*="language-"]': {
                  color: "#fff",
                },
              }}
            >
              {`cargo add listen-kit`}
            </SyntaxHighlighter>
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
      </div>
    </div>
  );
};

export default App;

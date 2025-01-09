import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";

import { useState } from "react";

const Header = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-gray-900/80 backdrop-blur border-b border-gray-800">
      <div className="max-w-6xl mx-auto px-4">
        <div className="flex justify-between items-center h-16">
          {/* Left side */}
          <div className="flex items-center space-x-4">
            <button
              className="md:hidden p-2"
              aria-label="Toggle navigation"
              onClick={toggleMenu}
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                className="w-6 h-6"
              >
                <path
                  strokeLinecap="round"
                  strokeWidth="2"
                  d="M4 6h16M4 12h16M4 18h16"
                />
              </svg>
            </button>

            <a href="/" className="flex items-center space-x-2">
              <img src="/listen.svg" alt="Logo" className="w-8 h-8" />
              <span className="font-bold text-xl">listen-rs</span>
            </a>
          </div>

          {/* Right side */}
          <div className="flex items-center space-x-4 hidden md:flex">
            <div className="items-center space-x-4">
              <a
                href="https://docs.listen-rs.com"
                className="text-gray-300 hover:text-white"
              >
                Documentation
              </a>
            </div>
            <a
              href="https://github.com/piotrostr/listen"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-300 hover:text-white flex items-center"
            >
              GitHub
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                className="ml-1"
                fill="currentColor"
              >
                <path d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z" />
              </svg>
            </a>
          </div>
        </div>

        {/* Mobile menu */}
        {isMenuOpen && (
          <div className="md:hidden">
            <div className="px-2 pt-2 pb-3 space-y-1 bg-gray-900/80 backdrop-blur">
              <a
                href="https://docs.listen-rs.com"
                className="block px-3 py-2 text-base font-medium text-gray-300 hover:text-white hover:bg-gray-700 rounded-md"
              >
                Documentation
              </a>
            </div>
            <div className="px-2 pt-2 pb-3 space-y-1 bg-gray-900/80 backdrop-blur mb-2 mt-2">
              <a
                href="https://github.com/piotrostr/listen"
                target="_blank"
                rel="noopener noreferrer"
                className="block px-3 py-2 text-base font-medium text-gray-300 hover:text-white hover:bg-gray-700 rounded-md flex flex-row items-center"
              >
                GitHub
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  className="ml-1"
                  fill="currentColor"
                >
                  <path d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z" />
                </svg>
              </a>
            </div>
          </div>
        )}
      </div>
    </nav>
  );
};

const App = () => {
  return (
    <div className="relative min-h-screen bg-black text-white">
      <Header />
      <div className="fixed inset-0 w-screen h-screen bg-[url('/bg.webp')] bg-cover bg-center opacity-10" />

      <div className="relative z-10 max-w-6xl mx-auto px-4 py-20">
        {/* Hero */}
        <div className="text-center mb-20 mt-10">
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
              {`cargo add --git https://github.com/piotrostr/listen listen`}
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

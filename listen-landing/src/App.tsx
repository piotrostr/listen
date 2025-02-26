import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Background } from "./Background";
import { Header } from "./Header";
import MultichainSwapDemo from "./MultichainSwapDemo";

const App = () => {
  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />

      {/* Main content with side-by-side layout */}
      <div className="container mx-auto px-4 mt-10 flex flex-col lg:flex-row">
        {/* Left side - Hero content */}
        <div className="lg:w-1/2 text-center lg:text-left lg:pr-8 mb-10 lg:mb-0">
          <img
            src="/listen-more.png"
            alt="listen"
            className="w-40 h-40 mx-auto lg:mx-0 mb-8 rounded shadow-lg"
          />
          <h1 className="text-5xl lg:text-6xl font-bold mb-6">listen</h1>
          <p className="text-xl text-gray-300 mb-6">
            blazingly fast actions for AI agents in Rust
          </p>
          <div className="mt-5 flex flex-row justify-center lg:justify-start items-center space-x-4 mb-6">
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
          <div className="mt-5 flex justify-center lg:justify-start text-lg [&>pre]:rounded-lg">
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

        {/* Right side - MultichainSwapDemo */}
        <div className="lg:w-1/2 flex items-center justify-center lg:justify-end">
          <div className="w-full max-w-xl border border-gray-700 rounded-lg overflow-hidden shadow-xl">
            <MultichainSwapDemo />
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;

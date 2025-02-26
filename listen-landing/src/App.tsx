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

      {/* Main content with grid layout */}
      <div className="container mx-auto px-4 mt-10 grid grid-cols-1 gap-4 min-[1000px]:grid-cols-2">
        {/* Left side - Hero content */}
        <div className="flex items-center justify-center min-h-[97vh] w-full py-6">
          <div className="flex flex-col h-full w-full items-center justify-center">
            <div className="max-w-md">
              <h1 className="text-2xl lg:text-5xl font-bold mb-6">
                Trade any token, <br />
                with words
              </h1>
              <p className="text-xl text-gray-300 mb-6">
                Listen is an AI portfolio manager agent
              </p>
              <div className="mt-5 flex flex-row items-center space-x-4 mb-6">
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
              {/* Actions Library Installation */}
              <div className="mt-5 text-lg [&>pre]:rounded-lg">
                <p className="text-sm text-gray-300 mb-2">
                  Install actions library:
                </p>
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

              {/* Try Early Access Button */}
              <div className="mt-8">
                <a
                  href="https://app.listen-rs.com"
                  className="px-6 py-3 text-white font-medium rounded-lg transition-colors duration-300 inline-flex items-center border border-purple-700 hover:border-purple-600"
                >
                  Try Early Access
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5 ml-2"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M10.293 5.293a1 1 0 011.414 0l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414-1.414L12.586 11H5a1 1 0 110-2h7.586l-2.293-2.293a1 1 0 010-1.414z"
                      clipRule="evenodd"
                    />
                  </svg>
                </a>
              </div>
            </div>
          </div>
        </div>

        {/* Right side - MultichainSwapDemo */}
        <div className="flex items-center min-h-[97vh] w-full py-6">
          <div className="w-full max-w-xl border border-gray-700 rounded-lg overflow-hidden shadow-xl">
            <MultichainSwapDemo />
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;

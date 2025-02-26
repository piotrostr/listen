import { useState } from "react";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";
import { Background } from "./Background";
import { Header } from "./Header";
import MultichainSwapDemo from "./MultichainSwapDemo";

const App = () => {
  const [activeOption, setActiveOption] = useState<"install" | "earlyAccess">(
    "earlyAccess"
  );

  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />

      {/* Main content with improved grid layout */}
      <div className="container mx-auto px-4 pt-20 md:pt-24 grid grid-cols-1 gap-8 lg:grid-cols-2 lg:gap-12">
        {/* Left side - Hero content */}
        <div className="flex items-center justify-center w-full py-4 md:py-6">
          <div className="flex flex-col w-full items-center justify-center">
            <div className="max-w-md lg:max-w-xl w-full">
              <h1 className="text-3xl lg:text-5xl font-bold mb-4 md:mb-6">
                Trade any token, <br />
                with words
              </h1>
              <div className="flex flex-row items-center space-x-4 mb-4 md:mb-6">
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

              {/* Option Toggle */}
              <div className="flex space-x-2 mb-4">
                <button
                  onClick={() => setActiveOption("earlyAccess")}
                  className={`px-4 py-2 rounded-lg transition-colors duration-200 ${
                    activeOption === "earlyAccess"
                      ? "bg-purple-700/40 text-white"
                      : "text-gray-400 hover:text-gray-300"
                  }`}
                >
                  For Users
                </button>
                <button
                  onClick={() => setActiveOption("install")}
                  className={`px-4 py-2 rounded-lg transition-colors duration-200 ${
                    activeOption === "install"
                      ? "bg-purple-700/40 text-white"
                      : "text-gray-400 hover:text-gray-300"
                  }`}
                >
                  For Developers
                </button>
              </div>

              {/* Conditional Content */}
              {activeOption === "install" ? (
                <div className="mt-3 md:mt-5 text-lg [&>pre]:rounded-lg animate-fadeIn">
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
              ) : (
                <div className="mt-3 md:mt-5 animate-fadeIn">
                  <a
                    href="https://app.listen-rs.com"
                    className="px-6 py-3 w-full text-center text-white font-medium rounded-lg transition-colors duration-300 inline-flex items-center justify-center border border-purple-700 hover:border-purple-600 bg-purple-900/30 hover:bg-purple-900/50"
                  >
                    Early Access
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
              )}
            </div>
          </div>
        </div>

        {/* Right side - MultichainSwapDemo */}
        <div className="flex items-center justify-center w-full">
          <div className="w-full max-w-xl lg:max-w-2xl rounded-lg overflow-hidden shadow-xl">
            <MultichainSwapDemo />
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;

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
        <MultichainSwapDemo />
      </div>
    </div>
  );
};

export default App;

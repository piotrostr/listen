import { usePrivy } from "@privy-io/react-auth";
import { FaExclamationTriangle } from "react-icons/fa";
import { PriceUpdates } from "./PriceUpdates";

export function GettingStarted() {
  const { login, ready } = usePrivy();

  return (
    <div className="flex flex-col items-center gap-4 p-2">
      {/* Getting Started Section */}
      <div className="w-full max-w-2xl mx-auto flex flex-col items-center text-center gap-2">
        <h2 className="text-xl lg:text-2xl font-bold mt-5 mb-2">
          how it works
        </h2>
        <p className="text-sm lg:text-base">
          listen is your single stop for on-chain trading with natural language
        </p>
        <p className="text-sm lg:text-base">
          1. create an account, you can use your email or wallet
        </p>
        <p className="text-sm lg:text-base">
          2. initialize a wallet for your AI agent, deposit funds and delegate
          access
        </p>
        <p className="text-sm lg:text-base">3. go wild!</p>
        <br />
        <button
          onClick={login}
          disabled={!ready}
          className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-purple-500/10"
        >
          get started
        </button>
        <p className="text-sm max-w-md text-center mt-3">
          should you have any questions - ask the agent directly - Listen
          understands the tools it has access to and has a view of the portfolio
          its managing
        </p>
      </div>

      <span className="text-sm my-8 mb-1 flex items-center gap-2">
        <FaExclamationTriangle className="text-yellow-500" />
        WARNING: listen is in early beta, things might not work as expected, use
        at own risk
      </span>

      {/* Divider */}
      <div className="border-t border-purple-500/30 w-full max-w-4xl" />

      {/* Screener Preview Section */}
      <div className="w-full max-w-4xl mx-auto">
        <div>
          <PriceUpdates />
        </div>
      </div>
    </div>
  );
}

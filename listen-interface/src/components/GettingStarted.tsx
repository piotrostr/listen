import { usePrivy } from "@privy-io/react-auth";
import { DelegateActionButton } from "./DelegateActionButton";

export function GettingStarted() {
  const { login, ready, user } = usePrivy();

  return (
    <div className="flex flex-col items-center gap-4">
      <h2 className="text-2xl font-bold">how it works</h2>
      <p>
        listen is your single stop for on-chain trading with natural language
      </p>
      <p>1. create an account, you can use your email or wallet</p>
      <p>
        2. initialize a wallet for your AI agent, deposit funds and delegate
        access
      </p>
      <p>3. go wild!</p>
      <br />
      {!user ? (
        <button
          onClick={login}
          disabled={!ready}
          className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-purple-500/10"
        >
          get started
        </button>
      ) : (
        <DelegateActionButton />
      )}
      <p className="text-sm max-w-md text-center">
        should you have any questions - ask the agent directly - Listen
        understands the tools it has access to and has a view of the portfolio
        its managing
      </p>
    </div>
  );
}

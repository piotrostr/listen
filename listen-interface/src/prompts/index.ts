import { CompactPortfolio } from "../hooks/util";
import { systemPromptEvm } from "./evm";
import { systemPromptSolana, systemPromptSolanaAgent } from "./solana";

export const pickSystemPrompt = (
  chatType: string,
  agentMode: boolean,
  portfolio: CompactPortfolio,
  defaultAmount: string,
  solanaWallet: string | null,
  evmWallet: string | null
) => {
  if (chatType === "evm") {
    return systemPromptEvm(portfolio, evmWallet, solanaWallet);
  }
  if (agentMode) {
    return systemPromptSolanaAgent(
      portfolio,
      solanaWallet || "",
      defaultAmount.toString()
    );
  }
  return systemPromptSolana(portfolio, solanaWallet, defaultAmount.toString());
};

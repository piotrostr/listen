import { CompactPortfolio } from "../hooks/util";
import { systemPromptEvm } from "./evm";
import { systemPromptSolana, systemPromptSolanaAgent } from "./solana";

export const pickSystemPrompt = (
  chatType: string,
  agentMode: boolean,
  portfolio: CompactPortfolio,
  defaultAmount: string,
  solanaWallet: string | null,
  evmWallet: string | null,
  isGuest: boolean
) => {
  if (chatType === "evm") {
    return systemPromptEvm(portfolio, evmWallet, solanaWallet);
  }
  if (agentMode) {
    const res = systemPromptSolanaAgent(
      portfolio,
      solanaWallet || "",
      defaultAmount.toString(),
      isGuest
    );
    return res;
  }
  const res = systemPromptSolana(
    portfolio,
    solanaWallet,
    defaultAmount.toString(),
    isGuest
  );
  return res;
};

import { PortfolioData } from "./types";

export function introPrompt(portfolio?: PortfolioData, userAddress?: string) {
  return `address: ${userAddress} ${JSON.stringify(portfolio)}`;
}

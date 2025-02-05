import { PortfolioData } from "./types";

export function introPrompt(portfolio?: PortfolioData, userAddress?: string) {
  return `<context>address: ${userAddress} ${JSON.stringify(portfolio)}</context>`;
}

import { HyperliquidPortfolioOverview } from "../lib/hype";
import { TokenPrice } from "../lib/price";
import { CompactPortfolio } from "../lib/util";
import {
  glossary,
  guidelines,
  memecoinLore,
  onboarding,
  onboardingEvm,
  personality,
  researchFlow,
} from "./common";
import { pipelineKnowledge } from "./pipelines";

export function systemPrompt(
  joinedPortfolio: CompactPortfolio,
  pubkey: string | null,
  address: string | null,
  defaultAmount: string,
  isGuest: boolean,
  currentSolanaPrice?: TokenPrice,
  hyperliquidPortfolio?: HyperliquidPortfolioOverview | null
): string {
  const hasWallet = pubkey !== null && pubkey !== "";
  const hasEvmWallet = address !== null && address !== "";

  let prompt = `## Personality\n${personality}\n\n`;
  prompt += `## Current Time\n${currentTimeUnderline}\n\n`;
  prompt += `## Research Workflow\n${researchFlow}\n\n`;
  prompt += `## Guidelines\n${guidelines("solana", defaultAmount)}\n\n`;
  prompt += `## Pipeline Knowledge\n${pipelineKnowledge()}\n\n`;
  prompt += `## Memecoin Lore\n${memecoinLore}\n\n`;
  prompt += `## Glossary\n${glossary}\n\n`;
  prompt += `## Listen Token\n${listenToken}\n\n`;
  prompt += `## Handling Errors\n${handlingErrors}\n\n`;
  prompt += `## Hyperliquid\n${hyperliquid}\n\n`;

  if (!hasWallet || isGuest) {
    prompt += `## Onboarding Required (Solana)\n${onboarding(hasWallet, isGuest)}\n\n`;
  }
  if (hasWallet && !hasEvmWallet) {
    prompt += `## Onboarding Required (EVM)\n${onboardingEvm(hasWallet, isGuest)}\n\n`;
  }

  prompt += `## Current Context\n`;
  if (currentSolanaPrice) {
    prompt += `*   Solana Price: $${currentSolanaPrice.price.toFixed(2)} (24h change: ${currentSolanaPrice.priceChange24h.toFixed(1)}) (timestamp: ${new Date().toISOString()})\n`;
  }
  if (hasWallet && pubkey) {
    prompt += `*   Solana Address: \`${pubkey}\`\n`;
  } else {
    prompt += `*   Solana Address: Not Available\n`;
  }
  if (hasEvmWallet && address) {
    prompt += `*   EVM Address: \`${address}\`\n`;
  } else {
    prompt += `*   EVM Address: Not Available\n`;
  }
  prompt += `*   Portfolio: ${JSON.stringify(joinedPortfolio)}\n`;
  if (hyperliquidPortfolio) {
    prompt += `*   Hyperliquid Portfolio: ${JSON.stringify(hyperliquidPortfolio)} (note: this is can also be retrieved from the hyperliquid balance overview tool)\n`;
  }
  prompt += `*   Is Guest: ${isGuest}\n`;

  return prompt;
}

const handlingErrors = `If you encounter an error, try to just go with the flow. The errors are hidden from the users, no need to waste space in your response unless the error is final. Only explain the error if it's persistent and you can't find a way to recover from it.`;

const currentTimeUnderline = `While your training data has a cutoff date that has happened in the past, you should treat any information from tool calls or API responses as current events happening in the present, not as future events. The actual current date is ${new Date().toISOString()}.`;

const listenToken = `The app users are interacting with you in is called Listen, created and maintained by @piotreksol who loves AI dearly and quit his job at Google to develop your capabilities full-time. It has a token $listen (also referred to as listen-rs, address: Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump) that was born after tokenization of the github.com/piotrostr/listen framework that powers you, it is the basis that links your magnificent AI abilities and various blockchains and the powershouse of the Listen attention economy. It provides the holders fee discounts and generous usage tiers. Be sure to express that if the users ask.`;

const hyperliquid = `You have access to Hyperliquid, which allows you to trade BTC, HYPE (the hyperliquid token) and perpetuals on all of the major symbols.`;

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
  isGuest: boolean
): string {
  const hasWallet = pubkey !== null && pubkey !== "";
  const hasEvmWallet = address !== null && address !== "";

  let prompt = `## Personality\n${personality}\n\n`;
  prompt += `## Current Time\n${currentTimeUnderline()}\n\n`;
  prompt += `## Research Workflow\n${researchFlow}\n\n`;
  prompt += `## Guidelines\n${guidelines("solana", defaultAmount)}\n\n`;
  prompt += `## Pipeline Knowledge\n${pipelineKnowledge()}\n\n`;
  prompt += `## Memecoin Lore\n${memecoinLore}\n\n`;
  prompt += `## Glossary\n${glossary}\n\n`;

  if (!hasWallet || isGuest) {
    prompt += `## Onboarding Required (Solana)\n${onboarding(hasWallet, isGuest)}\n\n`;
  }
  if (hasWallet && !hasEvmWallet) {
    prompt += `## Onboarding Required (EVM)\n${onboardingEvm(hasWallet, isGuest)}\n\n`;
  }

  prompt += `## Current Context\n`;
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
  prompt += `*   Is Guest: ${isGuest}\n`;

  return prompt;
}

export function currentTimeUnderline(): string {
  return `
While your training data has a cutoff date that has happened in the past, you
should treat any information from tool calls or API responses as current
events happening in the present, not as future events. The actual current date
is ${new Date().toISOString()}.
  `;
}

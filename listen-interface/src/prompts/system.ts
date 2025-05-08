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
import { miniapps } from "./miniapps";
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
  prompt += `## Reference Data\n\n### Memecoin Lore\n${memecoinLore}\n\n### Glossary\n${glossary}\n\n`;

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

export function worldchainPrompt(): string {
  let prompt =
    "You are a World Mini App companion. You can help users find and use World Mini Apps.\n\n";
  prompt +=
    "Help the user find the best World Mini App to suit their needs.\n\n";
  prompt += "Apps by category:\n";
  for (const [category, apps] of Object.entries(miniapps)) {
    prompt += `* ${category}\n`;
    for (const app of apps) {
      prompt += `  * name: ${app.name}\n`;
      prompt += `  * description: ${app.world_app_description}\n`;
      prompt += `  * logo_img_url: ${app.logo_img_url}\n`;
      prompt += `  * url: worldapp://mini-app?app_id=${app.app_id}\n`;
    }
  }
  prompt += `Be sure to provide the url for the apps in the format as you are given, the user will then be able to click on the url and open the app to be redirected straightaway.`;
  prompt += `## Current Time\n${currentTimeUnderline()}\n\n`;
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

import { CompactPortfolio } from "../hooks/util";
import {
  guidelines,
  onboarding,
  onboardingEvm,
  personality,
  researchFlow,
} from "./common";
import { pipelineKnowledge } from "./pipelines";

export function systemPrompt(
  joinedPortfolio: CompactPortfolio, // TODO ensure that the chain param is correct (caip2)
  pubkey: string | null,
  address: string | null,
  defaultAmount: string,
  isGuest: boolean
) {
  const hasWallet = pubkey !== null && pubkey !== "";
  const hasEvmWallet = address !== null && address !== "";
  return `
  ${personality}
  <current_time>${currentTimeUnderline()}</current_time>
  <research_flow>${researchFlow}</research_flow>
  <guidelines>${guidelines("solana", defaultAmount)}</guidelines>
  <knowledge>${pipelineKnowledge()}</knowledge>
  ${!hasWallet || isGuest ? `<onboarding>${onboarding(hasWallet, isGuest)}</onboarding>` : ""}
  ${hasWallet && pubkey ? `<solana_address>${pubkey}</solana_address>` : ""}
  ${hasEvmWallet ? `<evm_address>${address}</evm_address>` : onboardingEvm(hasWallet, isGuest)}
  <portfolio>${JSON.stringify(joinedPortfolio)}</portfolio>
  `;
}

export function currentTimeUnderline() {
  return `
  While your training data has a cutoff date that has happened in the past, you
  should treat any information from tool calls or API responses as current
  events happening in the present, not as future events. The actual current date
  is ${new Date().toISOString()}.
  `;
}

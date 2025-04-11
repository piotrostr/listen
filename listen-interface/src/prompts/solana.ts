import { addressBook, CompactPortfolio } from "../hooks/util";
import { guidelines, onboarding, personality, researchFlow } from "./common";
import { pipelineKnowledge } from "./pipelines";

const solanaErrors = "";

const solanaLimitations = `
Only discuss limitations if the user would ask about something you cannot do
- adding liquidity is currently not supported, JLP or mSOL are options you could suggest instead
- the research_x_profile could take as long as 30s to a minute to complete,
  mention to the user it might take around that time before you call it
`;

export function systemPromptSolana(
  solanaPortfolio: CompactPortfolio,
  pubkey: string | null,
  defaultAmount: string,
  isGuest: boolean
) {
  const hasWallet = pubkey !== null && pubkey !== "";
  return `
  ${personality}
  <guidelines>${guidelines("solana", defaultAmount)}</guidelines>
  <research_flow>${researchFlow}</research_flow>
  ${!hasWallet || isGuest ? `<onboarding>${onboarding(hasWallet, isGuest, "solana")}</onboarding>` : ""}
  <solana_address>${pubkey}</solana_address>
  <portfolio>Portfolio: ${JSON.stringify(solanaPortfolio)} (prices in USD)</portfolio>
  <address_book>Address book: ${JSON.stringify(addressBook["solana"])}</address_book>
  <knowledge>${pipelineKnowledge("solana")}</knowledge>
  <errors>${solanaErrors}</errors>
  <limitations>${solanaLimitations}</limitations>
  `;
}

export function systemPromptSolanaAgent(
  solanaPortfolio: CompactPortfolio,
  pubkey: string | null,
  defaultAmount: string,
  isGuest: boolean
) {
  const hasWallet = pubkey !== null && pubkey !== "";
  return `
  ${personality}
  <current_time>${currentTimeUnderline()}</current_time>
  <research_flow>${researchFlow}</research_flow>
  <guidelines>${guidelines("solana", defaultAmount)}</guidelines>
  ${!hasWallet || isGuest ? `<onboarding>${onboarding(hasWallet, isGuest, "solana")}</onboarding>` : ""}
  <solana_address>${pubkey}</solana_address>
  <portfolio>${JSON.stringify(solanaPortfolio)}</portfolio>
  <errors>${solanaErrors}</errors>
  `;
}

export function currentTimeUnderline() {
  return `
  While your training data has a cutoff date that has happened in the past, you
  should treat any information from tool calls or API responses as current
  events happening in the present, not as future events. The actual current date
  is ${new Date().toISOString().split("T")[0]}.
  `;
}

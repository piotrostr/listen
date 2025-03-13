import { addressBook, CompactPortfolio } from "../hooks/util";
import { guidelines, personality, personalityAgent } from "./common";
import { pipelineKnowledge } from "./pipelines";

const solanaErrors = `
0x1771: program error when slippage tolerance is too low, this can be fixed by increasing the slippage tolerance or a retry
`;

const solanaLimitations = `
Only discuss limitations if the user would ask about something you cannot do
- adding liquidity is currently not supported, jupiter liquidity proivder is an option you could suggest instead
- the research_x_profile could take as long as 30s to a minute to complete,
  mention to the user it might take around that time before you call it
`;

export function systemPromptSolana(
  solanaPortfolio: CompactPortfolio,
  pubkey: string | null,
  defaultAmount: string
) {
  return `
  <personality>${personality}</personality>
  <guidelines>${guidelines("solana", defaultAmount)}</guidelines>
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
  pubkey: string,
  defaultAmount: string
) {
  return `
  <personality>${personalityAgent}</personality>
  <guidelines>${guidelines("solana", defaultAmount, true)}</guidelines>
  <solana_address>${pubkey}</solana_address>
  <portfolio>${JSON.stringify(solanaPortfolio)}</portfolio>
  <errors>${solanaErrors}</errors>
  `;
}

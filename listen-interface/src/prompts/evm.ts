import { addressBook, caip2Map } from "../hooks/util";
import { guidelines, personality } from "./common";
import { pipelineKnowledge } from "./pipelines";

export function systemPromptEvm(
  portfolio: {
    chain: string;
    address: string;
    amount: string;
    name: string;
    symbol: string;
    decimals: number;
  }[],
  walletAddress: string | null,
  pubkey: string | null
) {
  return `
  <personality>${personality}</personality>
  <knowledge>${pipelineKnowledge("evm")}</knowledge>
  <guidelines>${guidelines("evm", undefined)}</guidelines>
  <evm_address>${walletAddress ?? "null"}</evm_address>
  <solana_address>${pubkey ?? "null"}</solana_address>
  <portfolio>Portfolio: ${JSON.stringify(portfolio)} (prices in USD)</portfolio>
  <chain_caip2_map>${JSON.stringify(caip2Map)}</chain_caip2_map>
  <address_book>${JSON.stringify(addressBook)}</address_book>
  `;
}

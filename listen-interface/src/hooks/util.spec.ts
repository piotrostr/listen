import { describe, expect, it } from "bun:test";
import { renderAddressOrTx } from "./util";

describe("renderAddressOrTx", () => {
  it("should render a solana transaction signature as a link", () => {
    const text = `I notice you haven't provided any specific question or request. Looking at our previous interaction, I successfully completed the USDC to BONK swap on Solana for you. The transaction was successful with the signature: "5CKCkU4P2YNGsRb6zYdYa9DbAK87wVbSHetvedXddkxGue9vsrA9yw3eUBEQXnNSsFfg7A7xorDVbvEoBPCrt6JH"
Is there anything else you'd like me to help you with? I can:
1. Check token prices
2. Help with another swap
3. Get quotes for different tokens
4. Help with cross-chain transfers

Just let me know what you'd like to do!`;
    const result = renderAddressOrTx(text);

    // Check if the result contains the link with the transaction signature
    expect(result).toContain(
      'href="https://solscan.io/tx/5CKCkU4P2YNGsRb6zYdYa9DbAK87wVbSHetvedXddkxGue9vsrA9yw3eUBEQXnNSsFfg7A7xorDVbvEoBPCrt6JH"'
    );
  });

  it("should handle multiple addresses in the same text", () => {
    const text =
      "Check these addresses: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v and 0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
    const result = renderAddressOrTx(text);

    expect(result).toContain(
      'href="https://solscan.io/address/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"'
    );
    expect(result).toContain(
      'href="https://etherscan.io/address/0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"'
    );
  });
});

import { describe, expect, it } from "bun:test";
import { embedResearchAnchors } from "../components/ResearchOutput";
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
      'href="https://blockscan.com/address/0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"'
    );
  });
});

describe("embedResearchAnchors", () => {
  it("should correctly convert Twitter references to numbered links", () => {
    const testMarkdown = `Based on the Twitter data, here's an analysis of Fartcoin ($FARTCOIN) and related discussions:

**Key Insight:** $FARTCOIN is a Solana-based meme coin that has garnered significant attention, with some users reporting substantial market cap achievements (over $1 Billion).

**Engagement Analysis:**

*   **High Visibility:** Mentions of Fartcoin have achieved high view counts, such as 457,678 (Viral) on @DegenerateNews/1869486422207725801.
*   **Moderate to High Engagement:** Several tweets about Fartcoin received moderate to high likes (100 - 2,209) and retweets (11 - 220), indicating growing community interest.
*   **Mixed Sentiment:** Some users are highly bullish (@firestarterSOL/1882199565836992729), while others are planning to short the coin (@Felixxx\_on\_sol/1898668765270909366), indicating polarized opinions.

**Core Messages:**

*   Fartcoin is presented as a disruptive meme coin on Solana, blending humor with AI. (@PredX\_AI/1900215382939754802)
*   It's being compared to other popular meme coins like $POPCAT, $BABYDOGE, and $GIGA. (@PredX\_AI/1900215382939754802)
*   Some users are manifesting future price increases for Fartcoin. (@goatdsalmon/1900305890630393958)
*   Whale activity is being monitored. (@SolWhaleFinder/1900305456762958049)

**Contract Addresses:**

*   Ar73TpP2Y2u1fP77u5ZWyUxZnpqx6fskguW6MYyxWgm (@Ruggy\_\_\_\_/1900307354656096422)
*   E3UDBuekE9ktwedTMb5iNhnFSxBAD484r2DMJLLxpump (@AlcoholiCrypto/1880685528452039162)

**Key References:**

*   @FartCoinOfSOL: Official Fartcoin account
*   @DegenerateNews: Reported on Fartcoin's market cap milestones (@DegenerateNews/1869486422207725801, @DegenerateNews/1866923884857135150)
*   @firestarterSOL: Bullish commentary and explanation of "FartStrategy" (@firestarterSOL/1882199565836992729)

**Future Analysis Points:**

*   Monitor the impact of "FartStrategy" on the price of Fartcoin.
*   Track whale activity and its correlation with price movements.
*   Analyze sentiment shifts (bullish vs. bearish) over time.
*   Further investigate the AI integration aspect mentioned by @PredX\_AI.`;

    const result = embedResearchAnchors(testMarkdown);

    // Check that all Twitter references are converted to links
    expect(result).not.toContain("@DegenerateNews/1869486422207725801");
    expect(result).not.toContain("@firestarterSOL/1882199565836992729");
    expect(result).not.toContain("@Felixxx_on_sol/1898668765270909366");
    expect(result).not.toContain("@PredX_AI/1900215382939754802");
    expect(result).not.toContain("@goatdsalmon/1900305890630393958");
    expect(result).not.toContain("@SolWhaleFinder/1900305456762958049");
    expect(result).not.toContain("@Ruggy____/1900307354656096422");
    expect(result).not.toContain("@AlcoholiCrypto/1880685528452039162");
    expect(result).not.toContain("@DegenerateNews/1866923884857135150");

    // Check that links are created with correct numbering
    expect(result).toContain(
      '<a href="https://x.com/DegenerateNews/status/1869486422207725801"'
    );
    expect(result).toContain(
      '<a href="https://x.com/firestarterSOL/status/1882199565836992729"'
    );
    expect(result).toContain(
      '<a href="https://x.com/Felixxx_on_sol/status/1898668765270909366"'
    );
    expect(result).toContain(
      '<a href="https://x.com/PredX_AI/status/1900215382939754802"'
    );
    expect(result).toContain(
      '<a href="https://x.com/Ruggy____/status/1900307354656096422"'
    );

    // Check that the same reference gets the same number
    const linkPattern =
      /<a href="https:\/\/x\.com\/PredX_AI\/status\/1900215382939754802"[^>]*>\[(\d+)\]<\/a>/g;
    const matches = [...result.matchAll(linkPattern)];
    expect(matches.length).toBeGreaterThan(1);
    expect(matches[0][1]).toBe(matches[1][1]); // Same reference number for both occurrences
  });
});

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

  it("should handle Twitter references in markdown without @ symbol", () => {
    const testMarkdown = `Here's a summary of the Solana meme coin ecosystem based on the provided tweets:

**General Sentiment:**

*   Solana meme coins are a hot topic, with many new coins launching frequently.
*   The market is described as "wrecked" and highly speculative, with some suggesting established cryptocurrencies like Bitcoin are safer bets (MSTR2100/1891822418999607785).
*   Some believe the "meme coin" phase might be losing steam in favor of Solana gaming projects (radioralo/1894386396174876731).

**Specific Meme Coins Mentioned:**

*   **$WIF, $BONK, $MYRO, $SAMO, $DUKO, $PONKE, $HARAMBE, $STAN, $WEN, $POPCAT, $LOAF, $CHONKY, $SMOG, $SILLY, $SOLAMA, $HONK, $PENG:**  These are listed as existing Solana meme coins (crypto\\_banter/1767669374217760953, CryptoAdam007/1767874726003646830).
*   **$MARS:** Aims to become a prominent meme coin across Ethereum, Solana, and BNB chains (Marserc20turkey/1892586696560034293).
*   **$DORAE:** Positioned as a meme coin with global appeal, based on the Doraemon character (DoraeCommunity/1895510818961432587).
*   **$GME:** Considered undervalued by some (earlymfer/1765875195233878211).
*   **$FWOG:** One user expresses comfort in investing significantly in this meme coin (lennoxxcartel/1899939377687708032).
*   **$WOULD:** Claims to be the second oldest meme coin on Solana (wouldmeme\\_sol/1899104892910031080).
*   **$PONGO:** A user hopes for a pump, even though the developer is gone (VividLotus\\_/1900240592463868310).
*   **$GECKO, $GM**: SolanaPrincess is adding those to the portfolio (SolanaPrincess/1739003820775186923)

**Notable Events & Trends:**

*   **Airdrops:** There's a mention of a potential Solana meme coin airdrop for Trump NFT/merch holders (ToBcryptonews/1892057912308040078).
*   **Giveaways:**  Users are running giveaways of SOL to promote their accounts (MrsolanaB/1817925629364391943).

**Key Accounts & Their Stance:**

*   **Crypto Banter & CryptoAdam007:** Listing various Solana meme coins, potentially for awareness or discussion.
*   **SolanaPrincess:** Actively trading meme coins and sharing their portfolio.
*   **barkmeta:** Launching a new SOL memecoin and soliciting Solana addresses.

**Engagement Levels:**

*   The tweet by Crypto Banter (crypto\\_banter/1767669374217760953) has a high engagement with 1.3k likes and 216k views.
*   The tweet by Bark (barkmeta/1797402785920487567) has a high engagement with 1.9k likes and 151k views.
*   The tweet by SPCM and Hobbes (SPCMNandHOBBES/1818781889282355394) has decent engagement with 2.5k likes and 200k views.`;

    const result = embedResearchAnchors(testMarkdown);

    // Check that Twitter references without @ are converted to links
    expect(result).toContain(
      '<a href="https://x.com/MSTR2100/status/1891822418999607785"'
    );
    expect(result).toContain(
      '<a href="https://x.com/radioralo/status/1894386396174876731"'
    );
    expect(result).toContain(
      '<a href="https://x.com/crypto_banter/status/1767669374217760953"'
    );
    expect(result).toContain(
      '<a href="https://x.com/CryptoAdam007/status/1767874726003646830"'
    );
    expect(result).toContain(
      '<a href="https://x.com/Marserc20turkey/status/1892586696560034293"'
    );
    expect(result).toContain(
      '<a href="https://x.com/DoraeCommunity/status/1895510818961432587"'
    );
    expect(result).toContain(
      '<a href="https://x.com/earlymfer/status/1765875195233878211"'
    );
    expect(result).toContain(
      '<a href="https://x.com/lennoxxcartel/status/1899939377687708032"'
    );
    expect(result).toContain(
      '<a href="https://x.com/wouldmeme_sol/status/1899104892910031080"'
    );
    expect(result).toContain(
      '<a href="https://x.com/VividLotus_/status/1900240592463868310"'
    );
    expect(result).toContain(
      '<a href="https://x.com/SolanaPrincess/status/1739003820775186923"'
    );
    expect(result).toContain(
      '<a href="https://x.com/ToBcryptonews/status/1892057912308040078"'
    );
    expect(result).toContain(
      '<a href="https://x.com/MrsolanaB/status/1817925629364391943"'
    );
    expect(result).toContain(
      '<a href="https://x.com/barkmeta/status/1797402785920487567"'
    );
    expect(result).toContain(
      '<a href="https://x.com/SPCMNandHOBBES/status/1818781889282355394"'
    );

    // Check that the same reference gets the same number
    const linkPattern =
      /<a href="https:\/\/x\.com\/crypto_banter\/status\/1767669374217760953"[^>]*>\[(\d+)\]<\/a>/g;
    const matches = [...result.matchAll(linkPattern)];
    expect(matches.length).toBe(2); // Now appears twice in the text
    expect(matches[0][1]).toBe(matches[1][1]); // Same reference number for both occurrences
  });
});

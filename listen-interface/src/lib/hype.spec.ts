import { describe, expect, it } from "bun:test";
import { Hyperliquid } from "./hype";

const USER_ADDRESS = "0xCCC48877a33a2C14e40c82da843Cf4c607ABF770";

describe("hype", () => {
  it.skip("should fetch portfolio overview", async () => {
    const hype = new Hyperliquid();
    const portfolioOverview = await hype.portfolioOverview(USER_ADDRESS);

    console.log(portfolioOverview);

    expect(portfolioOverview).toBeDefined();
  });

  it("should fetch spot balances", async () => {
    const hype = new Hyperliquid();
    const spotBalances = await hype.fetchSpotBalances(USER_ADDRESS);

    console.log(spotBalances);
  });

  it("should fetch perp balances", async () => {
    const hype = new Hyperliquid();
    const perpBalances = await hype.fetchPerpBalances(USER_ADDRESS);

    console.log(JSON.stringify(perpBalances, null, 2));
  });
});

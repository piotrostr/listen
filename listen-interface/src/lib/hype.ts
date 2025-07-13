import {
  ClearinghouseState,
  ClearinghouseStateSchema,
  SpotClearinghouseState,
  SpotClearinghouseStateSchema,
} from "./hype-types";

export interface HyperliquidPortfolioOverview {
  spotBalances: SpotClearinghouseState;
  perpBalances: ClearinghouseState;
}

export class Hyperliquid {
  readonly baseUrl = "https://api.hyperliquid.xyz/info";

  async portfolioOverview(
    address: string
  ): Promise<HyperliquidPortfolioOverview> {
    const spotBalances = await this.fetchSpotBalances(address);
    const perpBalances = await this.fetchPerpBalances(address);

    return {
      spotBalances,
      perpBalances,
    };
  }

  async fetchSpotBalances(address: string) {
    const response = await fetch(this.baseUrl, {
      method: "POST",
      body: JSON.stringify({
        type: "spotClearinghouseState",
        user: address,
      }),
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (!response.ok) {
      throw new Error(
        `Failed to fetch balances: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return SpotClearinghouseStateSchema.parse(data);
  }

  async fetchPerpBalances(address: string) {
    const response = await fetch(this.baseUrl, {
      method: "POST",
      body: JSON.stringify({
        type: "clearinghouseState",
        user: address,
      }),
      headers: {
        "Content-Type": "application/json",
      },
    });

    if (!response.ok) {
      throw new Error(
        `Failed to fetch balances: ${response.status} ${response.statusText}`
      );
    }

    const data = await response.json();
    return ClearinghouseStateSchema.parse(data);
  }
}

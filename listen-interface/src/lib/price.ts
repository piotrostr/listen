import { getNetworkId } from "./util";

export interface TokenPrice {
  price: number;
  priceChange24h: number;
}

export type TokenPriceMap = Map<string, TokenPrice>;

export interface GeckoTerminalResponse {
  data: {
    attributes: {
      token_prices: Record<string, string>;
      h24_price_change_percentage: Record<string, string>;
    };
  };
}

// Solana native token wrapped address
export const WSOL_MINT = "So11111111111111111111111111111111111111112";

// Group tokens by their network for batch fetching
function groupTokensByNetwork(
  tokens: { address: string; chain: string }[]
): Record<string, string[]> {
  return tokens.reduce(
    (acc, token) => {
      const networkId = getNetworkId(token.chain);
      if (!networkId) return acc;

      if (!acc[networkId]) {
        acc[networkId] = [];
      }
      acc[networkId].push(token.address);
      return acc;
    },
    {} as Record<string, string[]>
  );
}

async function fetchSolanaTokenPrice(mint: string): Promise<TokenPrice | null> {
  // For WSOL, use GeckoTerminal directly
  if (mint === WSOL_MINT) {
    return fetchSolanaTokenPriceFromGecko(mint);
  }

  try {
    // Try to get current price first
    const currentPriceResponse = await fetch(
      `https://api.listen-rs.com/v1/adapter/price?mint=${mint}`
    );

    // If current price request fails or returns null, consider token invalid
    if (!currentPriceResponse.ok) {
      return null;
    }

    const currentPriceData = await currentPriceResponse.json();
    if (!currentPriceData?.price) {
      return null;
    }

    // If we got current price, try to get 24h open price
    const openPriceResponse = await fetch(
      `https://api.listen-rs.com/v1/adapter/24h-open?mint=${mint}`
    );

    if (!openPriceResponse.ok) {
      // Return current price with 0 change if we can't get historical
      return {
        price: currentPriceData.price,
        priceChange24h: 0,
      };
    }

    const openPriceData = await openPriceResponse.json();
    if (!openPriceData?.price) {
      // Return current price with 0 change if historical data is invalid
      return {
        price: currentPriceData.price,
        priceChange24h: 0,
      };
    }

    // Calculate price change percentage
    const priceChange24h =
      ((currentPriceData.price - openPriceData.price) / openPriceData.price) *
      100;

    return {
      price: currentPriceData.price,
      priceChange24h,
    };
  } catch (error) {
    console.error(`Error fetching Solana token price for ${mint}:`, error);
    return null;
  }
}

async function fetchEvmNetworkPrices(
  network: string,
  addresses: string[]
): Promise<TokenPriceMap> {
  const response = await fetch(
    `https://api.geckoterminal.com/api/v2/simple/networks/${network}/token_price/${addresses.join(
      ","
    )}?include_24hr_price_change=true`,
    {
      headers: {
        accept: "application/json",
      },
    }
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch prices for network ${network}`);
  }

  const data = (await response.json()) as GeckoTerminalResponse;
  const result = new Map<string, TokenPrice>();

  Object.entries(data.data.attributes.token_prices).forEach(
    ([address, priceStr]) => {
      const price = parseFloat(priceStr);
      const priceChange = parseFloat(
        data.data.attributes.h24_price_change_percentage[address] || "0"
      );

      result.set(address, {
        price,
        priceChange24h: priceChange,
      });
    }
  );

  return result;
}

async function fetchSolanaTokenPriceFromGecko(
  mint: string
): Promise<TokenPrice | null> {
  try {
    const response = await fetch(
      `https://api.geckoterminal.com/api/v2/simple/networks/solana/token_price/${mint}?include_24hr_price_change=true`,
      {
        headers: {
          accept: "application/json",
        },
      }
    );

    if (!response.ok) {
      return null;
    }

    const data = (await response.json()) as GeckoTerminalResponse;
    const priceStr = data.data.attributes.token_prices[mint];
    const priceChange = parseFloat(
      data.data.attributes.h24_price_change_percentage[mint] || "0"
    );

    return {
      price: parseFloat(priceStr),
      priceChange24h: priceChange,
    };
  } catch (error) {
    console.error(
      `Error fetching Solana token price from Gecko for ${mint}:`,
      error
    );
    return null;
  }
}

export async function fetchTokenPrices(
  tokens: { address: string; chain: string }[]
): Promise<TokenPriceMap> {
  const priceMap = new Map<string, TokenPrice>();

  // Separate Solana and EVM tokens
  const solanaTokens = tokens.filter((token) => token.chain === "solana");
  const evmTokens = tokens.filter((token) => token.chain !== "solana");

  // Fetch Solana token prices
  await Promise.all(
    solanaTokens.map(async (token) => {
      try {
        // Try Listen API (or GeckoTerminal for WSOL)
        const price = await fetchSolanaTokenPrice(token.address);
        if (price) {
          priceMap.set(token.address, price);
        } else {
          // If price is null, token is likely invalid/scam, set price to 0
          priceMap.set(token.address, { price: 0, priceChange24h: 0 });
        }
      } catch (error) {
        console.error(
          `Failed to fetch Solana token price for ${token.address}:`,
          error
        );
        priceMap.set(token.address, { price: 0, priceChange24h: 0 });
      }
    })
  );

  // Fetch EVM token prices
  const tokensByNetwork = groupTokensByNetwork(evmTokens);
  await Promise.all(
    Object.entries(tokensByNetwork).map(async ([network, addresses]) => {
      try {
        const networkPrices = await fetchEvmNetworkPrices(network, addresses);
        networkPrices.forEach((price, address) => {
          priceMap.set(address, price);
        });
      } catch (error) {
        console.error(`Failed to fetch prices for network ${network}:`, error);
        // Set default values for failed tokens
        addresses.forEach((address) => {
          priceMap.set(address, { price: 0, priceChange24h: 0 });
        });
      }
    })
  );

  return priceMap;
}

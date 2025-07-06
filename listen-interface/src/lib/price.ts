import { z } from "zod";
import { config } from "../config";
import { getNetworkId } from "./util";

export interface TokenPrice {
  price: number;
  priceChange24h: number;
}

export type TokenPriceMap = Map<string, TokenPrice>;

export const GeckoTerminalResponseSchema = z.object({
  data: z.object({
    attributes: z.object({
      token_prices: z.record(z.string(), z.string().nullable()),
      h24_price_change_percentage: z.record(z.string(), z.string().nullable()),
    }),
  }),
});

export type GeckoTerminalResponse = z.infer<typeof GeckoTerminalResponseSchema>;

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

// @deprecated
// @ts-ignore: unused
async function fetchSolanaTokenPriceFromListenApi(
  mint: string
): Promise<TokenPrice | null> {
  // For WSOL, use GeckoTerminal directly
  if (mint === WSOL_MINT) {
    return fetchSolanaTokenPriceFromGecko(mint);
  }

  try {
    // Try to get current price first
    const currentPriceResponse = await fetch(
      `${config.adapterEndpoint}/price?mint=${mint}`
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

// Helper function to chunk array into batches
function chunkArray<T>(array: T[], size: number): T[][] {
  return Array.from({ length: Math.ceil(array.length / size) }, (_, i) =>
    array.slice(i * size, i * size + size)
  );
}

async function fetchEvmNetworkPrices(
  network: string,
  addresses: string[]
): Promise<TokenPriceMap> {
  const BATCH_SIZE = 30;
  const result = new Map<string, TokenPrice>();

  // Split addresses into batches of 30
  const batches = chunkArray(addresses, BATCH_SIZE);

  // Process each batch
  await Promise.all(
    batches.map(async (batchAddresses) => {
      try {
        const response = await fetch(
          `https://api.geckoterminal.com/api/v2/simple/networks/${network}/token_price/${batchAddresses.join(
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

        Object.entries(data.data.attributes.token_prices).forEach(
          ([address, priceStr]) => {
            const price = parseFloat(priceStr ?? "0");
            const priceChange = parseFloat(
              data.data.attributes.h24_price_change_percentage[address] ?? "0"
            );

            result.set(address, {
              price,
              priceChange24h: priceChange,
            });
          }
        );
      } catch (error) {
        console.error(
          `Failed to fetch prices for batch in network ${network}:`,
          error
        );
        // Set default values for failed batch
        batchAddresses.forEach((address) => {
          result.set(address, { price: 0, priceChange24h: 0 });
        });
      }
    })
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
    const priceChange =
      data.data.attributes.h24_price_change_percentage[mint] || "0";

    return {
      price: parseFloat(priceStr ?? "0"),
      priceChange24h: parseFloat(priceChange ?? "0"),
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

  if (solanaTokens.length > 0) {
    const solanaPrices = await fetchJupPrices(
      solanaTokens.map((token) => token.address)
    );
    solanaPrices.forEach((price, address) => {
      priceMap.set(address, price);
    });
  }

  // Fetch EVM token prices
  const tokensByNetwork = groupTokensByNetwork(evmTokens);
  if (Object.keys(tokensByNetwork).length > 0) {
    await Promise.all(
      Object.entries(tokensByNetwork).map(async ([network, addresses]) => {
        try {
          const networkPrices = await fetchEvmNetworkPrices(network, addresses);
          networkPrices.forEach((price, address) => {
            priceMap.set(address, price);
          });
        } catch (error) {
          console.error(
            `Failed to fetch prices for network ${network}:`,
            error
          );
          // Set default values for failed tokens
          addresses.forEach((address) => {
            priceMap.set(address, { price: 0, priceChange24h: 0 });
          });
        }
      })
    );
  }

  return priceMap;
}

const JupPrice = z.object({
  usdPrice: z.number().optional().nullable(),
  priceChange24h: z.number().optional().nullable(),
});

const JupPriceResponse = z.record(z.string(), JupPrice);

export const fetchJupPrices = async (
  addresses: string[]
): Promise<TokenPriceMap> => {
  const BATCH_SIZE = 50;
  const result = new Map<string, TokenPrice>();
  const batches = chunkArray(addresses, BATCH_SIZE);

  await Promise.all(
    batches.map(async (batchAddresses) => {
      try {
        const ids = batchAddresses.join(",");
        const response = await (
          await fetch(`https://lite-api.jup.ag/price/v3?ids=${ids}`)
        ).json();
        const parsed = JupPriceResponse.safeParse(response);

        if (!parsed.success) {
          console.error(
            `Error parsing Jup price response: ${ids}, raw: ${JSON.stringify(
              response
            )}, error: ${parsed.error}`
          );
          // Set default values for failed batch
          batchAddresses.forEach((address) => {
            result.set(address, { price: 0, priceChange24h: 0 });
          });
          return;
        }

        Object.entries(parsed.data).forEach(([address, price]) => {
          result.set(address, {
            price: price.usdPrice ?? 0,
            priceChange24h: price.priceChange24h ?? 0,
          });
        });
      } catch (error) {
        console.error("Error fetching Jup price batch:", error);
        // Set default values for failed batch
        batchAddresses.forEach((address) => {
          result.set(address, { price: 0, priceChange24h: 0 });
        });
      }
    })
  );

  return result;
};

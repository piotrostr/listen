import { useEffect, useState } from "react";
import { z } from "zod";
import { getNetworkId } from "../lib/util";
import { Spinner } from "./Spinner";

// Cache for pair addresses to prevent repeated API calls
const pairAddressCache: Record<string, { address: string; timestamp: number }> =
  {};
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes in milliseconds

export const PoolResponseSchema = z.object({
  data: z.array(
    z.object({
      attributes: z.object({
        address: z.string(),
        volume_usd: z.object({
          h24: z.string(),
        }),
      }),
    })
  ),
});

async function findPairAddress(
  tokenAddress: string,
  chainId: string,
  signal?: AbortSignal
): Promise<string | null> {
  // Check cache first
  const cacheKey = `${chainId}:${tokenAddress}`;
  const cached = pairAddressCache[cacheKey];
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
    return cached.address;
  }

  const network = getNetworkId(chainId);
  if (!network) {
    console.error(`Unsupported chain ID: ${chainId}`);
    return null;
  }

  console.debug(`Resolving pair address for ${tokenAddress} on ${network}`);

  try {
    const response = await fetch(
      `https://api.geckoterminal.com/api/v2/networks/${network}/tokens/${tokenAddress}/pools`,
      {
        headers: {
          Accept: "application/json;version=20230302",
        },
        signal,
      }
    );

    if (!response.ok) {
      throw new Error(`API request failed with status ${response.status}`);
    }

    const json = await response.json();
    const result = PoolResponseSchema.safeParse(json);

    if (!result.success) {
      console.error("Failed to parse API response:", result.error);
      return null;
    }

    // Sort pools by 24h volume to get the most active pool
    const sortedPools = result.data.data.sort(
      (a, b) =>
        parseFloat(b.attributes.volume_usd.h24) -
        parseFloat(a.attributes.volume_usd.h24)
    );

    // Get the first (highest volume) pool address if available
    if (sortedPools.length > 0) {
      const address = sortedPools[0].attributes.address;
      // Cache the result
      pairAddressCache[cacheKey] = {
        address,
        timestamp: Date.now(),
      };
      return address;
    }

    return null;
  } catch (error) {
    if (error instanceof Error && error.name === "AbortError") {
      // Ignore abort errors
      return null;
    }
    console.error("Failed to fetch pair address:", error);
    return null;
  }
}

export function GeckoTerminalChart({
  pairAddress,
  chainId,
  timeframe,
  tokenAddress,
}: {
  pairAddress?: string;
  tokenAddress?: string;
  chainId: string;
  timeframe: string;
}) {
  const [resolvedPairAddress, setResolvedPairAddress] = useState<
    string | undefined
  >(pairAddress);
  const [isResolving, setIsResolving] = useState(false);

  useEffect(() => {
    let mounted = true;
    const abortController = new AbortController();

    async function resolvePairAddress() {
      if (!tokenAddress) return;

      try {
        setIsResolving(true);
        const poolAddress = await findPairAddress(
          tokenAddress,
          chainId,
          abortController.signal
        );
        if (mounted) {
          setResolvedPairAddress(poolAddress || undefined);
        }
      } catch (error) {
        if (mounted) {
          console.error("Failed to resolve pair address:", error);
          setResolvedPairAddress(undefined);
        }
      } finally {
        if (mounted) {
          setIsResolving(false);
        }
      }
    }

    if (tokenAddress) {
      resolvePairAddress();
    } else {
      setResolvedPairAddress(pairAddress);
    }

    return () => {
      mounted = false;
      abortController.abort();
    };
  }, [tokenAddress, chainId, pairAddress]);

  if (!resolvedPairAddress && !tokenAddress) {
    console.error("Either pairAddress or tokenAddress must be provided");
    return null;
  }

  if (isResolving) {
    return (
      <div className="flex items-center justify-center h-full">
        <Spinner />
      </div>
    );
  }
  if (!resolvedPairAddress) {
    return null;
  }

  const network = getNetworkId(chainId);
  if (!network) {
    console.error(`Unknown chainId: ${chainId}`);
    return null;
  }

  const src = `https://www.geckoterminal.com/${network}/pools/${resolvedPairAddress}?embed=1&info=0&swaps=0&grayscale=0&light_chart=0&chart_type=price&resolution=${timeframe}`;

  return (
    <iframe
      height="100%"
      width="100%"
      title="GeckoTerminal Embed"
      src={src}
      allow="clipboard-write"
    />
  );
}

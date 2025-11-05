import { useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";
import { useAllWallets } from "./useAllWallets";

export function useInvalidatePortfolio() {
  const queryClient = useQueryClient();
  const { solanaAddress, evmAddress } = useAllWallets();

  const invalidatePortfolio = useCallback(
    async () => {
      const invalidations = [];

      // Invalidate portfolio queries for connected wallets
      if (solanaAddress) {
        invalidations.push(
          queryClient.invalidateQueries({
            queryKey: ["solana-portfolio", solanaAddress],
          })
        );
      }
      if (evmAddress) {
        invalidations.push(
          queryClient.invalidateQueries({
            queryKey: ["evm-portfolio", evmAddress],
          }),
          queryClient.invalidateQueries({
            queryKey: ["hyperliquid-portfolio", evmAddress],
          })
        );
      }

      await Promise.all(invalidations);
    },
    [queryClient, solanaAddress, evmAddress]
  );

  return { refreshPortfolio: invalidatePortfolio };
}
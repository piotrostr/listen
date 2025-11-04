import { useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";
import { useWalletStore } from "../store/walletStore";

export function useInvalidatePortfolio() {
  const queryClient = useQueryClient();
  const {
    solanaAddress,
    evmAddress,
    eoaSolanaAddress,
    eoaEvmAddress,
    activeWallet,
  } = useWalletStore();

  const invalidatePortfolio = useCallback(
    async (invalidateAll?: boolean) => {
      const invalidations = [];

      if (invalidateAll) {
        // Invalidate all portfolio queries
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
        if (eoaSolanaAddress) {
          invalidations.push(
            queryClient.invalidateQueries({
              queryKey: ["solana-portfolio", eoaSolanaAddress],
            })
          );
        }
        if (eoaEvmAddress) {
          invalidations.push(
            queryClient.invalidateQueries({
              queryKey: ["evm-portfolio", eoaEvmAddress],
            }),
            queryClient.invalidateQueries({
              queryKey: ["hyperliquid-portfolio", eoaEvmAddress],
            })
          );
        }
      } else {
        // Invalidate only active wallet queries
        const currentSolanaAddress =
          activeWallet === "listen"
            ? solanaAddress
            : activeWallet === "eoaSolana"
            ? eoaSolanaAddress
            : null;

        const currentEvmAddress =
          activeWallet === "listen"
            ? evmAddress
            : activeWallet === "eoaEvm"
            ? eoaEvmAddress
            : null;

        if (currentSolanaAddress) {
          invalidations.push(
            queryClient.invalidateQueries({
              queryKey: ["solana-portfolio", currentSolanaAddress],
            })
          );
        }
        if (currentEvmAddress) {
          invalidations.push(
            queryClient.invalidateQueries({
              queryKey: ["evm-portfolio", currentEvmAddress],
            }),
            queryClient.invalidateQueries({
              queryKey: ["hyperliquid-portfolio", currentEvmAddress],
            })
          );
        }
      }

      await Promise.all(invalidations);
    },
    [
      queryClient,
      solanaAddress,
      evmAddress,
      eoaSolanaAddress,
      eoaEvmAddress,
      activeWallet,
    ]
  );

  return { refreshPortfolio: invalidatePortfolio };
}
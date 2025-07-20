import { useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

export function usePortfolioInvalidation() {
  const queryClient = useQueryClient();

  const invalidatePortfolios = useCallback(async () => {
    // Invalidate all portfolio queries to refresh after transactions
    await queryClient.invalidateQueries({ 
      queryKey: ['solana-portfolio'] 
    });
    await queryClient.invalidateQueries({ 
      queryKey: ['evm-portfolio'] 
    });
    await queryClient.invalidateQueries({ 
      queryKey: ['hyperliquid-portfolio'] 
    });
  }, [queryClient]);

  return { invalidatePortfolios };
}
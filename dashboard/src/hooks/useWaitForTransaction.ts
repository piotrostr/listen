import { Connection, TransactionSignature } from "@solana/web3.js";
import { useQuery, useQueryClient } from "@tanstack/react-query";

interface TransactionResult {
  success: boolean;
  signature: TransactionSignature;
  error?: string;
}

export const useWaitForTransaction = (
  signature: TransactionSignature | null | undefined,
) => {
  const connection = new Connection(import.meta.env.VITE_RPC_URL);
  const queryClient = useQueryClient();

  return useQuery<TransactionResult>({
    queryKey: ["transaction", signature],
    queryFn: async (): Promise<TransactionResult> => {
      if (!signature) {
        throw new Error("No signature provided");
      }

      const maxAttempts = 10;
      const intervalMs = 1000; // 1 second

      for (let attempt = 0; attempt < maxAttempts; attempt++) {
        try {
          const status = await connection.getSignatureStatus(signature);

          if (status.value !== null) {
            if (status.value.err) {
              return {
                success: false,
                signature,
                error: "Transaction failed",
              };
            }

            if (
              status.value.confirmationStatus === "confirmed" ||
              status.value.confirmationStatus === "finalized"
            ) {
              queryClient.resetQueries({ queryKey: ["portfolio"] });
              queryClient.resetQueries({ queryKey: ["sol-balance"] });
              return {
                success: true,
                signature,
              };
            }
          }

          // If we haven't returned yet, wait before next attempt
          await new Promise((resolve) => setTimeout(resolve, intervalMs));
        } catch (error) {
          if (attempt === maxAttempts - 1) {
            return {
              success: false,
              signature,
              error:
                error instanceof Error
                  ? error.message
                  : "Unknown error occurred",
            };
          }
          // If not the last attempt, continue to next iteration
          await new Promise((resolve) => setTimeout(resolve, intervalMs));
        }
      }

      return {
        success: false,
        signature,
        error: "Transaction confirmation timeout",
      };
    },
    enabled: !!signature,
    retry: false,
    refetchOnWindowFocus: false,
  });
};

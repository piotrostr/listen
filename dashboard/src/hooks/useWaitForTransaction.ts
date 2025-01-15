import { Connection, TransactionSignature } from "@solana/web3.js";
import { useQueryClient, useMutation } from "@tanstack/react-query";

interface TransactionResult {
  success: boolean;
  signature: TransactionSignature;
  error?: string;
}

export const useWaitForTransaction = () => {
  const connection = new Connection("https://api.mainnet-beta.solana.com");
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (
      signature: TransactionSignature,
    ): Promise<TransactionResult> => {
      try {
        const latestBlockhash = await connection.getLatestBlockhash();

        const confirmation = await connection.confirmTransaction(
          {
            signature,
            ...latestBlockhash,
          },
          "confirmed",
        );

        if (confirmation.value.err) {
          return {
            success: false,
            signature,
            error: "Transaction failed",
          };
        }

        // refetch portfolio data
        queryClient.invalidateQueries({ queryKey: ["portfolio"] });

        return {
          success: true,
          signature,
        };
      } catch (error) {
        return {
          success: false,
          signature,
          error:
            error instanceof Error ? error.message : "Unknown error occurred",
        };
      }
    },
  });
};

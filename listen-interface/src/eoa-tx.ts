import { z } from "zod";
import { caip2ToLifiChainId } from "./hooks/util";
import { SwapOrderAction } from "./types/pipeline";

export const TransactionRequestSchema = z.object({
  to: z.string().optional(),
  from: z.string().optional(),
  nonce: z.number().optional(),
  gasLimit: z.string().optional(),
  gasPrice: z.string().optional(),
  data: z.string().optional(),
  value: z.string().optional(),
  chainId: z.number().optional(),
  type: z.number().optional(),
  accessList: z
    .array(
      z.object({
        address: z.string(),
        storageKeys: z.array(z.string()),
      })
    )
    .optional(),
  maxPriorityFeePerGas: z.string().optional(),
  maxFeePerGas: z.string().optional(),
  customData: z.record(z.any()).optional(),
  ccipReadEnabled: z.boolean().optional(),
});

export type TransactionRequest = z.infer<typeof TransactionRequestSchema>;

export async function swapStepToTransaction(
  swapAction: SwapOrderAction,
  signerAddress: string
): Promise<TransactionRequest | null> {
  try {
    const params = new URLSearchParams({
      fromChain: swapAction.from_chain_caip2
        ? caip2ToLifiChainId(swapAction.from_chain_caip2).toString()
        : "solana",
      toChain: swapAction.to_chain_caip2
        ? caip2ToLifiChainId(swapAction.to_chain_caip2).toString()
        : "solana",
      fromToken: swapAction.input_token,
      toToken: swapAction.output_token,
      fromAddress: signerAddress,
      toAddress: signerAddress,
      fromAmount: swapAction.amount,
    });

    const response = await fetch(`https://li.quest/v1/quote?${params}`);
    if (!response.ok) {
      throw new Error(`LI.FI API error: ${response.statusText}`);
    }

    const data = await response.json();
    return data.transactionRequest ?? null;
  } catch (error) {
    console.error(error);
    return null;
  }
}

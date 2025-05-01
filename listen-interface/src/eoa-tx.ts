import { getQuote } from "@lifi/sdk";
import { caip2ToLifiChainId } from "./hooks/util";
import type { SwapOrderAction } from "./types/pipeline";

export async function swapStepToTransaction(
  swapAction: SwapOrderAction,
  signerAddress: string
) {
  try {
    const quote = await getQuote({
      fromToken: swapAction.input_token,
      toToken: swapAction.output_token,
      fromAmount: swapAction.amount,
      fromAddress: signerAddress,
      toAddress: signerAddress,
      fromChain: swapAction.from_chain_caip2
        ? caip2ToLifiChainId(swapAction.from_chain_caip2)
        : "solana",
      toChain: swapAction.to_chain_caip2
        ? caip2ToLifiChainId(swapAction.to_chain_caip2)
        : "solana",
    });

    return quote.transactionRequest;
  } catch (error) {
    console.error(error);
  }
}

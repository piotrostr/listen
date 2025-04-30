import { getQuote, getTokenAllowance, setTokenAllowance } from "@lifi/sdk";
import type { Client } from "viem";
import { caip2ToLifiChainId } from "./hooks/util";
import type { SwapOrderAction } from "./types/pipeline";

const LIFI_DIAMOND = "0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE";

// TODO for solana <=> evm swaps, eoa for one of the chains is not sufficient
export async function swapStepToTransaction(
  swapAction: SwapOrderAction,
  signerAddress: string,
  walletClient: Client | null
) {
  console.log("swapStepToTransaction", swapAction, signerAddress);
  try {
    if (swapAction.from_chain_caip2?.startsWith("eip155:")) {
      if (!walletClient) {
        throw new Error("Wallet client not found");
      }

      const chainId = caip2ToLifiChainId(swapAction.from_chain_caip2);
      const allowanceData = await getTokenAllowance(
        {
          address: swapAction.input_token,
          chainId: chainId,
        },
        signerAddress as `0x${string}`,
        LIFI_DIAMOND
      );

      if (!allowanceData) {
        throw new Error("Allowance data not found");
      }

      if (allowanceData < BigInt(swapAction.amount)) {
        // Approve maximum possible amount (uint256 max)
        const MAX_UINT256 = BigInt(
          "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );

        await setTokenAllowance({
          walletClient: walletClient,
          token: {
            address: swapAction.input_token,
            chainId: chainId,
          },
          spenderAddress: LIFI_DIAMOND,
          amount: MAX_UINT256,
        });
      }
    }

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

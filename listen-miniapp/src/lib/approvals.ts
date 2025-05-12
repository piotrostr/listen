import { Address, type EIP1193Provider, Hash } from "viem";
import { SwapOrderAction } from "../types/pipeline";
import { TransactionRequest } from "./eoa-tx";

export const LIFI_DIAMOND = "0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE";

export const MAX_APPROVAL_AMOUNT =
  "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

export async function getAllowance(
  tokenAddress: string,
  ownerAddress: string,
  spenderAddress: string,
  provider: EIP1193Provider
): Promise<bigint> {
  const allowanceData = `0xdd62ed3e${ownerAddress.slice(2).padStart(64, "0")}${spenderAddress.slice(2).padStart(64, "0")}`;

  const result = await provider.request({
    method: "eth_call",
    params: [
      {
        to: tokenAddress as Address,
        data: allowanceData as Hash,
      },
      "latest",
    ],
  });

  return BigInt(result);
}

export async function createApprovalTransaction(
  tokenAddress: string,
  spenderAddress: string,
  fromAddress: string,
  provider: EIP1193Provider,
  chainId: string
) {
  // Estimate gas
  const approveData = `0x095ea7b3${spenderAddress.slice(2).padStart(64, "0")}${MAX_APPROVAL_AMOUNT}`;

  const gasLimit = await provider.request({
    method: "eth_estimateGas",
    params: [
      {
        from: fromAddress as Address,
        to: tokenAddress as Address,
        data: approveData as Hash,
        value: "0x0",
      },
      "latest",
    ],
  });

  const gasPrice = await provider.request({
    method: "eth_gasPrice",
  });

  const res: TransactionRequest = {
    from: fromAddress,
    to: tokenAddress,
    data: approveData,
    chainId: parseInt(chainId),
    gasLimit,
    gasPrice,
    value: "0x0",
  };

  return res;
}

export const ensureApprovals = async (
  swapAction: SwapOrderAction,
  signerAddress: string,
  provider: EIP1193Provider
): Promise<TransactionRequest | null> => {
  try {
    if (!swapAction.from_chain_caip2) {
      throw new Error("missing from_chain_caip2");
    }
    const allowanceData = await getAllowance(
      swapAction.input_token,
      signerAddress as `0x${string}`,
      LIFI_DIAMOND,
      provider
    );

    if (allowanceData < BigInt(swapAction.amount)) {
      return await createApprovalTransaction(
        swapAction.input_token,
        LIFI_DIAMOND,
        signerAddress,
        provider,
        swapAction.from_chain_caip2.split(":")[1]
      );
    }
  } catch (err) {
    console.error(err);
  }
  return null;
};

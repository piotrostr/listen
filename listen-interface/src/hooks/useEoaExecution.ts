import { useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { Connection, VersionedTransaction } from "@solana/web3.js";
import { MiniKit, type SendTransactionInput } from "@worldcoin/minikit-js";
import { type EIP1193Provider } from "viem";
import { ensureApprovals } from "../lib/approvals";
import { swapStepToTransaction } from "../lib/eoa-tx";
import { usePortfolioStore } from "../store/portfolioStore";
import { SwapOrderAction } from "../types/pipeline";
import { waitForTransaction } from "../utils/transactionMonitor";

const PERMIT2_PROXY_ADDRESS = "0xA3C7a31a2A97b847D967e0B755921D084C46a742";

export function useEoaExecution() {
  const { wallets: evmWallets } = useWallets();
  const { wallets: solanaWallets } = useSolanaWallets();
  const { refreshPortfolio } = usePortfolioStore();

  const handleEoaSolana = async (
    action: SwapOrderAction,
    eoaSolanaAddress: string
  ): Promise<string | null> => {
    try {
      const wallet = solanaWallets.find((w) => w.address === eoaSolanaAddress);
      if (wallet) {
        const tx = await swapStepToTransaction(action, eoaSolanaAddress);
        if (!tx) {
          throw new Error("Failed to create Solana transaction request");
        }
        const transaction = VersionedTransaction.deserialize(
          Uint8Array.from(Buffer.from(tx?.data ?? "", "base64"))
        );
        const rpcUrl =
          import.meta.env.VITE_RPC_URL || "https://api.mainnet-beta.solana.com";
        const connection = new Connection(rpcUrl);
        const res = await wallet.sendTransaction(transaction, connection);
        await waitForTransaction(res, rpcUrl, () => {
          refreshPortfolio(true);
        });
        return res;
      }
    } catch (error) {
      console.error(error);
    }

    return null;
  };

  const handleEoaEvm = async (
    action: SwapOrderAction,
    eoaEvmAddress: string
  ): Promise<string | null> => {
    try {
      const wallet = evmWallets.find((w) => w.address === eoaEvmAddress);
      if (wallet) {
        const provider = await wallet.getEthereumProvider();
        const approvalsTx = await ensureApprovals(
          action,
          eoaEvmAddress,
          provider as unknown as EIP1193Provider
        );
        if (approvalsTx) {
          await provider.request({
            method: "eth_sendTransaction",
            params: [approvalsTx],
          });
          refreshPortfolio(true);
        }
        const tx = await swapStepToTransaction(action, eoaEvmAddress);
        if (!tx) {
          throw new Error("Failed to create EVM transaction request");
        }
        const res = await provider.request({
          method: "eth_sendTransaction",
          params: [tx],
        });
        // TODO add evm tx monitor, can use the builtin provider from privy and polling
        await new Promise((resolve) => setTimeout(resolve, 2000));
        refreshPortfolio(true);
        return res;
      }
    } catch (error) {
      console.error(error);
    }

    return null;
  };

  const handleEoaWorld = async (
    action: SwapOrderAction,
    worldAddress: string
  ): Promise<string | null> => {
    try {
      if (!MiniKit.isInstalled()) {
        throw new Error("World App is not installed");
      }

      const tx = await swapStepToTransaction(action, worldAddress);
      if (!tx || !tx.to) {
        throw new Error("Failed to create World transaction request");
      }

      const permit2 = {
        permitted: {
          token: action.input_token,
          amount: action.amount,
        },
        spender: PERMIT2_PROXY_ADDRESS,
        nonce: Date.now().toString(),
        deadline: Math.floor((Date.now() + 30 * 60 * 1000) / 1000).toString(), // 30 minutes
      };
      const calldata = tx.data;
      if (!calldata) {
        throw new Error(
          "Failed to create World transaction request: missing calldata from LiFi response"
        );
      }

      const txInput: SendTransactionInput = {
        transaction: [
          {
            address: PERMIT2_PROXY_ADDRESS,
            abi: [
              {
                inputs: [
                  { name: "calldata", type: "bytes" },
                  {
                    name: "permitData",
                    type: "tuple",
                    components: [
                      { name: "token", type: "address" },
                      { name: "amount", type: "uint256" },
                      { name: "nonce", type: "uint256" },
                      { name: "deadline", type: "uint256" },
                    ],
                  },
                  { name: "signature", type: "bytes" },
                ],
                name: "callDiamondWithPermit2",
                outputs: [],
                stateMutability: "nonpayable",
                type: "function",
              },
            ],
            functionName: "callDiamondWithPermit2",
            args: [
              calldata,
              [
                [action.input_token, action.amount],
                permit2.nonce,
                permit2.deadline,
              ],
              "PERMIT2_SIGNATURE_PLACEHOLDER_0",
            ],
          },
        ],
        permit2: [permit2],
      };

      console.log(txInput);

      const { finalPayload } =
        await MiniKit.commandsAsync.sendTransaction(txInput);

      console.log(finalPayload);

      if (finalPayload.status === "error") {
        throw new Error(
          finalPayload.details
            ? JSON.stringify(finalPayload.details)
            : `Transaction failed: ${JSON.stringify(finalPayload)}`
        );
      }

      // Wait for transaction confirmation
      await new Promise((resolve) => setTimeout(resolve, 2000));
      refreshPortfolio(true);

      return finalPayload.transaction_id;
    } catch (error) {
      console.error(error);
    }

    return null;
  };

  return {
    handleEoaSolana,
    handleEoaEvm,
    handleEoaWorld,
  };
}

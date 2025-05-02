import { useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { Connection, VersionedTransaction } from "@solana/web3.js";
import { type EIP1193Provider } from "viem";
import { ensureApprovals } from "../approvals";
import { swapStepToTransaction } from "../eoa-tx";
import { usePortfolioStore } from "../store/portfolioStore";
import { SwapOrderAction } from "../types/pipeline";
import { waitForTransaction } from "../utils/transactionMonitor";

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

  return {
    handleEoaSolana,
    handleEoaEvm,
  };
}

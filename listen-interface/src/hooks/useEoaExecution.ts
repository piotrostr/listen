import { useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { Connection, VersionedTransaction } from "@solana/web3.js";
import { type EIP1193Provider } from "viem";
import { ensureApprovals } from "../approvals";
import { swapStepToTransaction } from "../eoa-tx";
import { usePortfolioStore } from "../store/portfolioStore";
import { SwapOrderAction } from "../types/pipeline";

export function useEoaExecution() {
  const { wallets: evmWallets } = useWallets();
  const { wallets: solanaWallets } = useSolanaWallets();
  const { refreshPortfolio } = usePortfolioStore();

  const handleEoaSolana = async (
    action: SwapOrderAction,
    eoaSolanaAddress: string
  ): Promise<boolean> => {
    try {
      const wallet = solanaWallets.find((w) => w.address === eoaSolanaAddress);
      if (wallet) {
        const tx = await swapStepToTransaction(action, eoaSolanaAddress);
        const transaction = VersionedTransaction.deserialize(
          Uint8Array.from(Buffer.from(tx?.data ?? "", "base64"))
        );
        const connection = new Connection(
          import.meta.env.VITE_SOLANA_RPC_URL ||
            "https://api.mainnet-beta.solana.com"
        );
        await wallet.sendTransaction(transaction, connection);
        refreshPortfolio(true);
        return true;
      }
    } catch (error) {
      console.error(error);
      return false;
    }
    return false;
  };

  const handleEoaEvm = async (
    action: SwapOrderAction,
    eoaEvmAddress: string
  ): Promise<boolean> => {
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
        await provider.request({
          method: "eth_sendTransaction",
          params: [tx],
        });
        return true;
      }
    } catch (error) {
      console.error(error);
    }
    return false;
  };

  return {
    handleEoaSolana,
    handleEoaEvm,
  };
}

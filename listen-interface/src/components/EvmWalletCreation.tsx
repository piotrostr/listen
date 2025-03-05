import {
  useCreateWallet,
  useDelegatedActions,
  usePrivy,
  useWallets,
  type WalletWithMetadata,
} from "@privy-io/react-auth";
import { useState } from "react";

interface EvmWalletCreationProps {
  index?: number;
  status?: "Pending" | "Completed" | "Failed" | "Cancelled";
  error: string | null;
}

export const EvmWalletCreation = ({
  index,
  status,
  error,
}: EvmWalletCreationProps) => {
  const { user } = usePrivy();
  const { ready: evmReady, wallets: evmWallets } = useWallets();
  const { createWallet: createEvmWallet } = useCreateWallet();
  const { delegateWallet } = useDelegatedActions();

  const [isCreating, setIsCreating] = useState(false);

  const onCreateWallet = async () => {
    try {
      setIsCreating(true);
      await createEvmWallet();
    } catch (error) {
      console.error("Error creating EVM wallet:", error);
    } finally {
      setIsCreating(false);
    }
  };

  // Find EVM embedded wallet to delegate
  const evmWalletToDelegate = evmWallets.find(
    (wallet) => wallet.walletClientType === "privy"
  );

  // Check delegation status for EVM
  const isEvmDelegated = user?.linkedAccounts.some(
    (account): account is WalletWithMetadata =>
      account.type === "wallet" &&
      account.delegated &&
      account.chainType === "ethereum"
  );

  if (evmReady && !evmWalletToDelegate) {
    return (
      <button
        disabled={!evmReady || isCreating}
        onClick={onCreateWallet}
        className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-purple-500/10"
      >
        {isCreating ? (
          <span>Creating EVM wallet...</span>
        ) : (
          <span>Create EVM wallet</span>
        )}
      </button>
    );
  }

  if (!isEvmDelegated && evmWalletToDelegate) {
    return (
      <button
        disabled={!evmReady || !evmWalletToDelegate}
        onClick={async () => {
          try {
            await delegateWallet({
              address: evmWalletToDelegate.address,
              chainType: "ethereum",
            });
          } catch (error) {
            console.error("Error delegating EVM wallet:", error);
          }
        }}
        className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-purple-500/10"
      >
        Delegate EVM
      </button>
    );
  }

  return (
    <div className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm">
      {error ? (
        <span className="text-red-500">Error: {error}</span>
      ) : isEvmDelegated ? (
        <span className="text-green-500">EVM wallet delegated</span>
      ) : (
        <span>No EVM wallet available</span>
      )}
    </div>
  );
};

import {
  useDelegatedActions,
  useLogin,
  usePrivy,
  useSolanaWallets,
  type WalletWithMetadata,
} from "@privy-io/react-auth";
import { useState } from "react";
import { FaCheckCircle } from "react-icons/fa";

interface SolanaWalletCreationProps {
  error: string | null;
}

export const SolanaWalletCreation = ({ error }: SolanaWalletCreationProps) => {
  const { user } = usePrivy();
  const {
    ready: solanaReady,
    wallets: solanaWallets,
    createWallet: createSolanaWallet,
  } = useSolanaWallets();
  const { delegateWallet } = useDelegatedActions();
  const { login } = useLogin();

  const [isCreating, setIsCreating] = useState(false);
  const [isSignup, setIsSignup] = useState(false);
  const onCreateWallet = async () => {
    try {
      setIsCreating(true);
      await createSolanaWallet();
    } catch (error) {
      console.error("Error creating Solana wallet:", error);
    } finally {
      setIsCreating(false);
    }
  };

  const handleSignup = () => {
    try {
      setIsSignup(true);
      login();
    } catch (error) {
      console.error("Error signing up:", error);
    } finally {
      setIsSignup(false);
    }
  };

  if (user?.isGuest) {
    return (
      <button
        disabled={isSignup}
        onClick={handleSignup}
        className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-[#2D2D2D]"
      >
        {isSignup ? "Signing up..." : "Sign up"}
      </button>
    );
  }

  // Find Solana embedded wallet to delegate
  const solanaWalletToDelegate = solanaWallets.find(
    (wallet) => wallet.walletClientType === "privy"
  );

  // Check delegation status for Solana
  const isSolanaDelegated = user?.linkedAccounts.some(
    (account): account is WalletWithMetadata =>
      account.type === "wallet" &&
      account.delegated &&
      account.chainType === "solana"
  );

  if (solanaReady && !solanaWalletToDelegate) {
    return (
      <button
        disabled={!solanaReady || isCreating}
        onClick={onCreateWallet}
        className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-[#2D2D2D]"
      >
        {isCreating ? (
          <span>Creating Solana wallet...</span>
        ) : (
          <span>Create Solana wallet</span>
        )}
      </button>
    );
  }

  if (!isSolanaDelegated && solanaWalletToDelegate) {
    return (
      <button
        disabled={!solanaReady || !solanaWalletToDelegate}
        onClick={async () => {
          try {
            await delegateWallet({
              address: solanaWalletToDelegate.address,
              chainType: "solana",
            });
          } catch (error) {
            console.error("Error delegating Solana wallet:", error);
          }
        }}
        className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-[#2D2D2D]"
      >
        Delegate Solana
      </button>
    );
  }

  return (
    <div className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm">
      {error ? (
        <span className="text-red-500">Error: {error}</span>
      ) : isSolanaDelegated ? (
        <>
          <FaCheckCircle className="text-green-500 mr-2" />
          <span className="text-green-500">Solana wallet delegated</span>
        </>
      ) : (
        <span>No Solana wallet available</span>
      )}
    </div>
  );
};

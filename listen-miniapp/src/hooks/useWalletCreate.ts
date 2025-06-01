import { useDelegatedActions, useSolanaWallets } from "@privy-io/react-auth";
import { useState } from "react";
import { usePanel } from "../contexts/PanelContext";
import { useIsAuthenticated } from "./useIsAuthenticated";
import { usePrivyWallets } from "./usePrivyWallet";

export const useWalletCreate = () => {
  const [isCreating, setIsCreating] = useState(false);
  const { setActivePanel } = usePanel();
  const { ready: solanaReady, createWallet: createSolanaWallet } =
    useSolanaWallets();
  const { delegateWallet } = useDelegatedActions();
  const { solanaWalletAddress } = usePrivyWallets();
  const { hasSolanaWallet: hasSolanaWalletDelegated } = useIsAuthenticated();

  const handleCreate = async () => {
    if (!solanaReady || isCreating) return;
    try {
      setIsCreating(true);
      // no wallet - create
      if (solanaReady && !solanaWalletAddress) {
        await createSolanaWallet();
        // wallet - not delegated - delegate
      } else if (solanaWalletAddress && !hasSolanaWalletDelegated) {
        await delegateWallet({
          address: solanaWalletAddress,
          chainType: "solana",
        });
      } else {
        // wallet - delegated - fund
        setActivePanel("fund");
      }
    } catch (error) {
      console.error("Error in wallet creation/delegation:", error);
    } finally {
      setIsCreating(false);
    }
  };

  const getButtonText = () => {
    if (!solanaReady) return "Loading...";
    if (isCreating) return "Creating...";
    if (!solanaWalletAddress) return "Create wallet";
    if (!hasSolanaWalletDelegated) return "Delegate wallet";
    return "Fund wallet";
  };

  return {
    handleCreate,
    getButtonText,
    isCreating,
    solanaReady,
    solanaWalletAddress,
    hasSolanaWalletDelegated,
  };
};

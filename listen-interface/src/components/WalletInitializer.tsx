import { usePrivy, useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { useEffect, useRef } from "react";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";

/**
 * This component handles synchronization between Privy wallet state and wallet store
 */
export function WalletInitializer() {
  const { user } = usePrivy();
  const { ready: solanaReady, wallets: solanaWallets } = useSolanaWallets();
  const { ready: evmReady, wallets: evmWallets } = useWallets();
  const {
    setWalletAddresses,
    solanaAddress: currentSolanaAddress,
    evmAddress: currentEvmAddress,
  } = useWalletStore();
  const { initializePortfolioManager } = usePortfolioStore();

  // Use refs to track previous values and initialization status
  const prevSolanaAddressRef = useRef(currentSolanaAddress);
  const prevEvmAddressRef = useRef(currentEvmAddress);
  const initializedRef = useRef(false);

  // Effect to synchronize Privy wallets with our store
  useEffect(() => {
    if (!solanaReady || !evmReady || !user) return;

    // Ensure portfolio manager is initialized once when ready
    if (!initializedRef.current) {
      console.debug("WalletInitializer: Initializing portfolio manager");
      initializePortfolioManager();
      initializedRef.current = true;
    }

    // Find the Privy wallets
    const solanaWallet = solanaWallets.find(
      (wallet) =>
        wallet.type === "solana" && wallet.walletClientType === "privy"
    );

    const evmWallet = evmWallets.find(
      (wallet) =>
        wallet.type === "ethereum" && wallet.walletClientType === "privy"
    );

    const newSolanaAddress = solanaWallet?.address ?? null;
    const newEvmAddress = evmWallet?.address ?? null;

    // Update wallet addresses in the store if they have changed
    if (
      newSolanaAddress !== prevSolanaAddressRef.current ||
      newEvmAddress !== prevEvmAddressRef.current
    ) {
      console.debug(
        "WalletInitializer: Wallet addresses changed, updating store"
      );
      // Update refs to new values
      prevSolanaAddressRef.current = newSolanaAddress;
      prevEvmAddressRef.current = newEvmAddress;

      // Update the store
      setWalletAddresses(newSolanaAddress, newEvmAddress);
    }
  }, [
    solanaReady,
    evmReady,
    user,
    solanaWallets, // re-run if wallets change
    evmWallets, // re-run if wallets change
    setWalletAddresses,
    initializePortfolioManager,
  ]);

  // This component doesn't render anything
  return null;
}

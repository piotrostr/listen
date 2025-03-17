import { usePrivy, useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { useEffect, useRef } from "react";
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

  // Use refs to track previous values
  const prevSolanaAddressRef = useRef(currentSolanaAddress);
  const prevEvmAddressRef = useRef(currentEvmAddress);

  // Effect to synchronize Privy wallets with our store
  useEffect(() => {
    if (!solanaReady || !evmReady || !user) return;

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

    // Only update if addresses have actually changed
    if (
      newSolanaAddress !== prevSolanaAddressRef.current ||
      newEvmAddress !== prevEvmAddressRef.current
    ) {
      // Update refs to new values
      prevSolanaAddressRef.current = newSolanaAddress;
      prevEvmAddressRef.current = newEvmAddress;

      // Update the store
      setWalletAddresses(newSolanaAddress, newEvmAddress);
    }
  }, [
    solanaReady,
    evmReady,
    solanaWallets,
    evmWallets,
    setWalletAddresses,
    user,
  ]);

  // This component doesn't render anything
  return null;
}

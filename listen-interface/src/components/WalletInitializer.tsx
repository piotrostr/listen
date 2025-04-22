import { usePrivy, useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { useEffect, useRef } from "react";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";

export function WalletInitializer() {
  const { user } = usePrivy();
  const { ready: solanaReady, wallets: solanaWallets } = useSolanaWallets();
  const { ready: evmReady, wallets: evmWallets } = useWallets();
  const {
    setWalletAddresses,
    solanaAddress: currentSolanaAddress,
    evmAddress: currentEvmAddress,
  } = useWalletStore();
  const { initializePortfolioManager, fetchAllPortfolios } =
    usePortfolioStore();

  const prevSolanaAddressRef = useRef(currentSolanaAddress);
  const prevEvmAddressRef = useRef(currentEvmAddress);
  const initializedRef = useRef(false);
  const initialFetchDoneRef = useRef(false);

  // Effect for initial portfolio fetch - runs only once when wallets are ready
  useEffect(() => {
    if (!solanaReady || !evmReady || !user || initialFetchDoneRef.current)
      return;

    // Mark as done first to prevent subsequent runs
    initialFetchDoneRef.current = true;

    console.debug("WalletInitializer: Performing initial portfolio fetch");
    fetchAllPortfolios();
  }, [solanaReady, evmReady, user, fetchAllPortfolios]);

  // Effect for wallet address changes and initialization
  useEffect(() => {
    // Exit early if dependencies not ready
    if (!solanaReady || !evmReady || !user) return;

    // Initialize portfolio manager (only once)
    if (!initializedRef.current) {
      console.debug("WalletInitializer: Initializing portfolio manager");
      initializePortfolioManager();
      initializedRef.current = true;
    }

    // Find current wallet addresses
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

    // Only check for changes when needed
    const addressesChanged =
      newSolanaAddress !== prevSolanaAddressRef.current ||
      newEvmAddress !== prevEvmAddressRef.current;

    if (addressesChanged) {
      console.debug(
        "WalletInitializer: Wallet addresses changed, updating store"
      );
      prevSolanaAddressRef.current = newSolanaAddress;
      prevEvmAddressRef.current = newEvmAddress;

      setWalletAddresses(newSolanaAddress, newEvmAddress);
      fetchAllPortfolios();
    }
  }, [
    // Only depend on the specific wallet properties we need
    solanaReady,
    evmReady,
    user,
    // Use object reference equality for collections instead of deep equality
    solanaWallets,
    evmWallets,
    setWalletAddresses,
    initializePortfolioManager,
    fetchAllPortfolios,
  ]);

  return null;
}

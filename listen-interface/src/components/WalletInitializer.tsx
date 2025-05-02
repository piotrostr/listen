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
    eoaSolanaAddress: currentEoaSolanaAddress,
    eoaEvmAddress: currentEoaEvmAddress,
    eoaEvmIcon: currentEoaEvmIcon,
    eoaSolanaIcon: currentEoaSolanaIcon,
    setEoaSolanaAddress,
    setEoaEvmAddress,
    setEoaEvmIcon,
    setEoaSolanaIcon,
  } = useWalletStore();
  const { initializePortfolioManager, fetchAllPortfolios } =
    usePortfolioStore();

  const prevSolanaAddressRef = useRef(currentSolanaAddress);
  const prevEvmAddressRef = useRef(currentEvmAddress);
  const prevEoaSolanaAddressRef = useRef(currentEoaSolanaAddress);
  const prevEoaEvmAddressRef = useRef(currentEoaEvmAddress);
  const prevEoaEvmIconRef = useRef(currentEoaEvmIcon);
  const prevEoaSolanaIconRef = useRef(currentEoaSolanaIcon);
  const initializedRef = useRef(false);
  const initialFetchDoneRef = useRef(false);

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
    const newAddresses = {
      solana:
        solanaWallets.find(
          (w) => w.type === "solana" && w.walletClientType === "privy"
        )?.address ?? null,
      evm:
        evmWallets.find(
          (w) => w.type === "ethereum" && w.walletClientType === "privy"
        )?.address ?? null,
      eoaSolana:
        solanaWallets.find(
          (w) => w.type === "solana" && w.walletClientType !== "privy"
        )?.address ?? null,
      eoaEvm:
        evmWallets.find(
          (w) => w.type === "ethereum" && w.walletClientType !== "privy"
        )?.address ?? null,
    };

    const newIcons = {
      eoaEvm:
        evmWallets.find(
          (w) => w.type === "ethereum" && w.walletClientType !== "privy"
        )?.meta?.icon ?? null,
      eoaSolana:
        solanaWallets.find(
          (w) => w.type === "solana" && w.walletClientType !== "privy"
        )?.meta?.icon ?? null,
    };

    // Check what specifically changed
    const changes = {
      listen:
        newAddresses.solana !== prevSolanaAddressRef.current ||
        newAddresses.evm !== prevEvmAddressRef.current,
      eoaSolana: newAddresses.eoaSolana !== prevEoaSolanaAddressRef.current,
      eoaEvm: newAddresses.eoaEvm !== prevEoaEvmAddressRef.current,
      icons:
        newIcons.eoaEvm !== prevEoaEvmIconRef.current ||
        newIcons.eoaSolana !== prevEoaSolanaIconRef.current,
    };

    const anyChanges = Object.values(changes).some(Boolean);

    if (anyChanges || !initialFetchDoneRef.current) {
      // Update refs first
      prevSolanaAddressRef.current = newAddresses.solana;
      prevEvmAddressRef.current = newAddresses.evm;
      prevEoaSolanaAddressRef.current = newAddresses.eoaSolana;
      prevEoaEvmAddressRef.current = newAddresses.eoaEvm;
      prevEoaEvmIconRef.current = newIcons.eoaEvm;
      prevEoaSolanaIconRef.current = newIcons.eoaSolana;

      // Batch updates by type
      Promise.resolve()
        .then(() => {
          // Only update what changed
          if (changes.listen) {
            setWalletAddresses(newAddresses.solana, newAddresses.evm);
          }
          if (changes.eoaSolana) {
            setEoaSolanaAddress(newAddresses.eoaSolana);
          }
          if (changes.eoaEvm) {
            setEoaEvmAddress(newAddresses.eoaEvm);
          }
          if (changes.icons) {
            setEoaEvmIcon(newIcons.eoaEvm);
            setEoaSolanaIcon(newIcons.eoaSolana);
          }
        })
        .then(() => {
          if (!initialFetchDoneRef.current) {
            console.debug(
              "WalletInitializer: Performing initial portfolio fetch"
            );
            fetchAllPortfolios(true);
            initialFetchDoneRef.current = true;
          } else if (anyChanges) {
            console.debug(
              "WalletInitializer: Wallet addresses changed, updating portfolios"
            );
            fetchAllPortfolios(false);
          }
        });
    }
  }, [solanaReady, evmReady, user, solanaWallets, evmWallets]);

  return null;
}

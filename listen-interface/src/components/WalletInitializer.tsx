import { usePrivy, useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { useEffect, useRef } from "react";
import { useWalletStore } from "../store/walletStore";

export function WalletInitializer() {
  const { user } = usePrivy();
  const { ready: solanaReady, wallets: solanaWallets } = useSolanaWallets();
  const { ready: evmReady, wallets: evmWallets } = useWallets();
  const {
    setWalletAddresses,
    setEoaSolanaAddress,
    setEoaEvmAddress,
    setEoaEvmIcon,
    setEoaSolanaIcon,
  } = useWalletStore();

  const initializedRef = useRef(false);

  useEffect(() => {
    // Exit early if dependencies not ready
    if (!solanaReady || !evmReady || !user || initializedRef.current) return;

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

    // Set wallet addresses and icons
    setWalletAddresses(newAddresses.solana, newAddresses.evm);
    setEoaSolanaAddress(newAddresses.eoaSolana);
    setEoaEvmAddress(newAddresses.eoaEvm);
    setEoaEvmIcon(newIcons.eoaEvm);
    setEoaSolanaIcon(newIcons.eoaSolana);

    initializedRef.current = true;
  }, [solanaReady, evmReady, user, solanaWallets, evmWallets]);

  return null;
}

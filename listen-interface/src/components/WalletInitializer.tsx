import { usePrivy, useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { useEffect } from "react";
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
    setEoaEvmWallets,
  } = useWalletStore();

  useEffect(() => {
    // Exit early if dependencies not ready
    if (!solanaReady || !evmReady || !user) return;

    // Find current wallet addresses
    const newAddresses = {
      solana:
        solanaWallets.find(
          (w) => w.type === "solana" && w.walletClientType === "privy",
        )?.address ?? null,
      evm:
        evmWallets.find(
          (w) => w.type === "ethereum" && w.walletClientType === "privy",
        )?.address ?? null,
      eoaSolana:
        solanaWallets.find(
          (w) => w.type === "solana" && w.walletClientType !== "privy",
        )?.address ?? null,
      eoaEvm:
        evmWallets.find(
          (w) => w.type === "ethereum" && w.walletClientType !== "privy",
        )?.address ?? null,
    };

    const newIcons = {
      eoaEvm:
        evmWallets.find(
          (w) => w.type === "ethereum" && w.walletClientType !== "privy",
        )?.meta?.icon ?? null,
      eoaSolana:
        solanaWallets.find(
          (w) => w.type === "solana" && w.walletClientType !== "privy",
        )?.meta?.icon ?? null,
    };

    // Get all EOA EVM wallets (non-Privy)
    const allEoaEvmWallets = evmWallets
      .filter((w) => w.type === "ethereum" && w.walletClientType !== "privy")
      .map((w) => ({
        address: w.address,
        icon: w.meta?.icon ?? null,
        name: w.meta?.name || w.walletClientType || "Unknown Wallet",
      }));

    // Set wallet addresses and icons
    setWalletAddresses(newAddresses.solana, newAddresses.evm);
    setEoaSolanaAddress(newAddresses.eoaSolana);
    setEoaEvmAddress(newAddresses.eoaEvm);
    setEoaEvmIcon(newIcons.eoaEvm);
    setEoaSolanaIcon(newIcons.eoaSolana);
    setEoaEvmWallets(allEoaEvmWallets);
  }, [
    solanaReady,
    evmReady,
    user,
    solanaWallets,
    evmWallets,
    setWalletAddresses,
    setEoaSolanaAddress,
    setEoaEvmAddress,
    setEoaEvmIcon,
    setEoaSolanaIcon,
    setEoaEvmWallets,
  ]);

  return null;
}

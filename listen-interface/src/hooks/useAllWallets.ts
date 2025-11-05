import { useSolanaWallets, useWallets } from "@privy-io/react-auth";

/**
 * Ultra-simple hook to access connected EOA wallets.
 *
 * Browser manages which account is active (MetaMask account switch, Phantom account switch).
 * We just use the first wallet from each chain - that's always the active one.
 *
 * No wallet preference store needed - browser handles everything!
 */
export function useAllWallets() {
  const { wallets: evmWallets, ready: evmReady } = useWallets();
  const { wallets: solanaWallets, ready: solanaReady } = useSolanaWallets();

  // First wallet = active wallet (browser-managed)
  const evmWallet = evmWallets[0] ?? null;
  const solanaWallet = solanaWallets[0] ?? null;

  return {
    // Current active wallets
    evmWallet,
    solanaWallet,

    // Convenience addresses
    evmAddress: evmWallet?.address ?? null,
    solanaAddress: solanaWallet?.address ?? null,

    // Connection status
    hasEvmWallet: evmWallets.length > 0,
    hasSolanaWallet: solanaWallets.length > 0,
    hasBothWallets: evmWallets.length > 0 && solanaWallets.length > 0,

    // All wallets (for multi-wallet scenarios if needed later)
    evmWallets,
    solanaWallets,

    // Ready states
    ready: evmReady && solanaReady,
  };
}

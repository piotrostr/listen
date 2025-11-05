import { useAllWallets } from "../hooks/useAllWallets";

const iconOrPlaceholder = (icon: string | null | undefined) => {
  if (!icon) {
    return "/icons/placeholder.png";
  }
  return icon;
};

/**
 * Displays connected EOA wallets (EVM and/or Solana).
 * No switching logic needed - browser manages active accounts.
 * Just shows what's currently connected.
 */
export const WalletSwitcher = () => {
  const { evmWallet, solanaWallet } = useAllWallets();

  // If no wallets connected, don't show anything
  if (!evmWallet && !solanaWallet) {
    return null;
  }

  return (
    <div className="flex items-center justify-center gap-2 mb-4">
      {evmWallet && (
        <div className="p-2 rounded-lg bg-[#2D2D2D] text-white">
          <img
            src={iconOrPlaceholder(evmWallet.meta?.icon)}
            alt="EVM Wallet"
            className="w-6 h-6 rounded-full"
            title={`${evmWallet.meta?.name || "EVM Wallet"}: ${evmWallet.address.slice(0, 6)}...${evmWallet.address.slice(-4)}`}
          />
        </div>
      )}

      {solanaWallet && (
        <div className="p-2 rounded-lg bg-[#2D2D2D] text-white">
          <img
            src={iconOrPlaceholder(solanaWallet.meta?.icon)}
            alt="Solana Wallet"
            className="w-6 h-6 rounded-full"
            title={`${solanaWallet.meta?.name || "Solana Wallet"}: ${solanaWallet.address.slice(0, 6)}...${solanaWallet.address.slice(-4)}`}
          />
        </div>
      )}
    </div>
  );
};

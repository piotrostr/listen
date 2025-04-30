import { useWalletStore } from "../store/walletStore";

const iconOrPlaceholder = (icon: string | null) => {
  if (!icon) {
    return "/icons/placeholder.png";
  }
  return icon;
};

export const WalletSwitcher = () => {
  const {
    eoaSolanaAddress,
    eoaEvmAddress,
    eoaEvmIcon,
    eoaSolanaIcon,
    activeWallet,
    setActiveWallet,
  } = useWalletStore();

  return (
    <div className="flex items-center justify-center gap-2 mb-4">
      <button
        onClick={() => setActiveWallet("listen")}
        className={`p-2 rounded-lg transition-all ${
          activeWallet === "listen"
            ? "bg-[#2D2D2D] text-white"
            : "bg-black/40 text-gray-400"
        }`}
      >
        <img
          src="/listen-new.svg"
          alt="Listen"
          className="w-6 h-6 rounded-full"
        />
      </button>

      {eoaEvmAddress && (
        <button
          onClick={() => setActiveWallet("eoaEvm")}
          className={`p-2 rounded-lg transition-all ${
            activeWallet === "eoaEvm"
              ? "bg-[#2D2D2D] text-white"
              : "bg-black/40 text-gray-400"
          }`}
        >
          <img
            src={iconOrPlaceholder(eoaEvmIcon)}
            alt="EVM"
            className="w-6 h-6 rounded-full"
          />
        </button>
      )}

      {eoaSolanaAddress && (
        <button
          onClick={() => setActiveWallet("eoaSolana")}
          className={`p-2 rounded-lg transition-all ${
            activeWallet === "eoaSolana"
              ? "bg-[#2D2D2D] text-white"
              : "bg-black/40 text-gray-400"
          }`}
        >
          <img
            src={iconOrPlaceholder(eoaSolanaIcon)}
            alt="Solana"
            className="w-6 h-6 rounded-full"
          />
        </button>
      )}
    </div>
  );
};

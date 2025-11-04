import { useWalletStore } from "../store/walletStore";

export function EoaEvmWalletSelector() {
  const { eoaEvmWallets, selectedEoaEvmIndex, setSelectedEoaEvmIndex } = useWalletStore();

  if (eoaEvmWallets.length <= 1) {
    return null; // Don't show selector if there's only one or no wallets
  }

  return (
    <div className="mb-4">
      <label className="block text-sm font-medium text-gray-300 mb-2">
        Select EVM Wallet
      </label>
      <select
        value={selectedEoaEvmIndex}
        onChange={(e) => setSelectedEoaEvmIndex(Number(e.target.value))}
        className="w-full bg-[#2D2D2D] border border-gray-600 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        {eoaEvmWallets.map((wallet, index) => (
          <option key={wallet.address} value={index}>
            {wallet.name} - {wallet.address.slice(0, 6)}...{wallet.address.slice(-4)}
          </option>
        ))}
      </select>
    </div>
  );
}
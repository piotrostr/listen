import { useFundWallet } from "@privy-io/react-auth/solana";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { FaApplePay, FaExchangeAlt, FaShoppingCart } from "react-icons/fa";
import { IoArrowDown } from "react-icons/io5";
import { useChatType } from "../hooks/useChatType";
import { useEvmPortfolio } from "../hooks/useEvmPortfolioAlchemy";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { useSolanaPortfolio } from "../hooks/useSolanaPortfolio";
import { imageMap } from "../hooks/util";
import { BuySellModal } from "./BuySellModal";
import { CopyIcon } from "./CopyIcon";
import { PortfolioSkeleton } from "./PortfolioSkeleton";

export function Portfolio() {
  // const { fundWallet } = useFundWallet();
  const { data: solanaAssets, isLoading: isLoadingSolana } =
    useSolanaPortfolio();
  const { data: evmAssets, isLoading: isLoadingEvm } = useEvmPortfolio();
  const { data: wallets } = usePrivyWallets();
  const { chatType } = useChatType(); // Get the global chat type from settings

  // Local state for chain display toggle
  const [displayChain, setDisplayChain] = useState<"solana" | "ethereum">(() =>
    chatType === "solana" ? "solana" : "ethereum"
  );

  const [clickedAddress, setClickedAddress] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [modalAction, setModalAction] = useState<"buy" | "sell">("buy");
  const [selectedAsset, setSelectedAsset] = useState<any>(null);
  const { fundWallet } = useFundWallet();

  // Use local display chain instead of directly using chatType
  const selectedChain = displayChain;
  const isLoading = selectedChain === "solana" ? isLoadingSolana : isLoadingEvm;
  const displayedAssets =
    selectedChain === "solana" ? (solanaAssets ?? []) : (evmAssets ?? []);

  const { t } = useTranslation();

  if (isLoading) {
    return <PortfolioSkeleton />;
  }

  const handleClickCopy = () => {
    if (!wallets) return;
    const address =
      selectedChain === "solana"
        ? (wallets?.solanaWallet?.toString() ?? "")
        : (wallets?.evmWallet?.toString() ?? "");

    if (address) {
      navigator.clipboard.writeText(address);
      setClickedAddress(true);
      setTimeout(() => setClickedAddress(false), 1000);
    }
  };

  const handleToggleChain = () => {
    setDisplayChain((prev) => (prev === "solana" ? "ethereum" : "solana"));
  };

  const handleTopup = async () => {
    await fundWallet(wallets!.solanaWallet!);
  };

  const handleOpenModal = (asset: any, action: "buy" | "sell") => {
    setSelectedAsset(asset);
    setModalAction(action);
    setModalOpen(true);
  };

  const currentAddress =
    selectedChain === "solana"
      ? wallets?.solanaWallet?.toString()
      : wallets?.evmWallet?.toString();

  return (
    <div className="h-full font-mono  overflow-y-auto scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
      <div className="flex flex-row justify-between items-center p-4 lg:mt-3 lg:mb-3">
        <h2 className="text-xl font-bold lg:mb-0 mb-2">
          {t("portfolio.title")}
        </h2>

        {/* Address Display with Chain Toggle */}
        <div className="flex items-center gap-2">
          {currentAddress && (
            <>
              <img
                src={imageMap[selectedChain]}
                alt={selectedChain}
                className="w-4 h-4 rounded-full"
              />
              {currentAddress.slice(0, 4)}...
              <div onClick={handleClickCopy} className="cursor-pointer">
                {clickedAddress ? <div> ✅</div> : <CopyIcon />}
              </div>
            </>
          )}
          {/* Enhanced Chain Toggle Button with Switch Icon */}
          <div
            onClick={handleToggleChain}
            className="cursor-pointer ml-2 px-2 py-1 rounded-lg hover:bg-purple-500/20 transition-colors flex items-center gap-1"
            title={`Switch to ${selectedChain === "solana" ? "Ethereum" : "Solana"} assets`}
          >
            <img
              src={imageMap[selectedChain === "solana" ? "eth" : "solana"]}
              alt={
                selectedChain === "solana"
                  ? "Switch to Ethereum"
                  : "Switch to Solana"
              }
              className="w-5 h-5 rounded-full"
            />
            <FaExchangeAlt className="text-purple-300 text-sm" />
          </div>
        </div>
      </div>

      <div className="flex-1">
        <div className="p-4 pt-0 space-y-4">
          {displayedAssets
            .sort((a, b) => b.price * b.amount - a.price * a.amount)
            .map((asset) => (
              <div
                key={`${asset.address}-${asset.chain}`}
                className="border border-purple-500/30 rounded-lg p-3 hover:bg-purple-900/20 transition-colors"
              >
                <div className="flex justify-between items-start mb-2">
                  <div className="flex items-center gap-3">
                    {asset.logoURI ? (
                      <img
                        src={asset.logoURI}
                        alt={asset.symbol}
                        className="w-8 h-8 rounded-full"
                      />
                    ) : (
                      <div className="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center">
                        <span className="text-gray-500 dark:text-gray-400">
                          ?
                        </span>
                      </div>
                    )}
                    <div>
                      <h3 className="font-bold flex items-center gap-2">
                        {asset.name}{" "}
                        {/* TODO unified portfolio makes sense to display chain, otherwise hidden */}
                        <img
                          src={
                            "https://dd.dexscreener.com/ds-data/chains/" +
                            asset.chain.toLowerCase() +
                            ".png"
                          }
                          className="w-4 h-4 hidden"
                          alt={asset.chain}
                        />
                      </h3>
                      <p className="text-sm text-gray-400">${asset.symbol}</p>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="flex items-center gap-2">
                      {selectedChain === "solana" &&
                        asset.address ===
                          "So11111111111111111111111111111111111111112" && (
                          <button
                            className="cursor-pointer border border-purple-500/30 rounded-full p-2 bg-purple-500/10 hover:bg-purple-500/20 transition-colors"
                            onClick={handleTopup}
                          >
                            <FaApplePay size={32} />
                          </button>
                        )}
                      <div>
                        <p className="font-bold">
                          ${(asset.price * asset.amount).toFixed(2)}
                        </p>
                        <p className="text-sm text-gray-400">
                          $
                          {asset.address !==
                          "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
                            ? asset.price?.toFixed(6)
                            : asset.price?.toFixed(2)}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
                <div className="flex justify-between items-center">
                  <div className="text-sm text-gray-400">
                    {t("portfolio.holding")}: {asset.amount}
                  </div>

                  {/* Buy/Sell buttons - only show for Solana chain assets */}
                  {selectedChain === "solana" && (
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleOpenModal(asset, "buy")}
                        className="px-2 py-1 bg-green-500/20 hover:bg-green-500/30 text-green-300 border border-green-500/30 rounded-lg text-xs transition-colors flex items-center gap-1"
                      >
                        <FaShoppingCart size={12} />
                        <span>{t("portfolio.buy")}</span>
                      </button>
                      <button
                        onClick={() => handleOpenModal(asset, "sell")}
                        className="px-2 py-1 bg-red-500/20 hover:bg-red-500/30 text-red-300 border border-red-500/30 rounded-lg text-xs transition-colors flex items-center gap-1"
                      >
                        <IoArrowDown size={12} />
                        <span>{t("portfolio.sell")}</span>
                      </button>
                    </div>
                  )}
                </div>
              </div>
            ))}
          {displayedAssets.length === 0 && (
            <div className="text-center text-gray-400">
              {t("portfolio.no_assets_found")}
            </div>
          )}
        </div>
      </div>

      {/* Buy/Sell Modal */}
      {modalOpen && selectedAsset && (
        <BuySellModal
          isOpen={modalOpen}
          onClose={() => setModalOpen(false)}
          action={modalAction}
          asset={selectedAsset}
        />
      )}
    </div>
  );
}

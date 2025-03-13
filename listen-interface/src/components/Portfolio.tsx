import { useFundWallet } from "@privy-io/react-auth/solana";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { FaApplePay, FaShoppingCart, FaSync } from "react-icons/fa";
import { IoArrowDown } from "react-icons/io5";
import { useModal } from "../contexts/ModalContext";
import { useSettings } from "../contexts/SettingsContext";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { usePortfolioStore } from "../store/portfolioStore";
import { BuySellModal } from "./BuySellModal";
import { PortfolioSkeleton } from "./PortfolioSkeleton";

export function Portfolio() {
  // Get chatType from useChatType hook
  const { chatType } = useSettings();

  // Use the portfolio store
  const {
    combinedPortfolio: assets,
    isLoading,
    refreshPortfolio,
    isFresh,
    setChatType: setStoreChatType,
  } = usePortfolioStore();

  // Keep portfolio store chatType in sync with global chatType
  useEffect(() => {
    setStoreChatType(chatType);
  }, [chatType, setStoreChatType]);

  const { data: wallets } = usePrivyWallets();
  const { t } = useTranslation();
  const { openChart } = useModal();
  const { fundWallet } = useFundWallet();

  const [modalOpen, setModalOpen] = useState(false);
  const [modalAction, setModalAction] = useState<"buy" | "sell">("buy");
  const [selectedAsset, setSelectedAsset] = useState<any>(null);

  // Determine which wallet addresses to use based on chatType
  const getSolanaAddress = () => wallets?.solanaWallet || "";
  const getEvmAddress = () =>
    chatType === "solana" ? "" : wallets?.evmWallet || "";

  // Initial fetch - only if not fresh
  useEffect(() => {
    if (
      (wallets?.solanaWallet || (wallets?.evmWallet && chatType === "omni")) &&
      !isFresh()
    ) {
      refreshPortfolio(getSolanaAddress(), getEvmAddress());
    }
  }, [
    wallets?.solanaWallet,
    wallets?.evmWallet,
    isFresh,
    refreshPortfolio,
    chatType,
  ]);

  // Focus detection - must be in the same position in the hooks order
  useEffect(() => {
    // Function to handle visibility change
    const handleVisibilityChange = () => {
      if (
        document.visibilityState === "visible" &&
        (wallets?.solanaWallet || (wallets?.evmWallet && chatType === "omni"))
      ) {
        // On becoming visible, refresh if needed
        refreshPortfolio(getSolanaAddress(), getEvmAddress());
      }
    };

    // Add visibility listener
    document.addEventListener("visibilitychange", handleVisibilityChange);

    // Clean up
    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [wallets?.solanaWallet, wallets?.evmWallet, refreshPortfolio, chatType]);

  const handleTopup = async () => {
    await fundWallet(wallets!.solanaWallet!);
  };

  const handleOpenModal = (asset: any, action: "buy" | "sell") => {
    setSelectedAsset(asset);
    setModalAction(action);
    setModalOpen(true);
  };

  const handleRefresh = async () => {
    console.log("handleRefresh");
    // Force refresh
    await refreshPortfolio(getSolanaAddress(), getEvmAddress(), true);
  };

  if (isLoading && (!assets || assets.length === 0)) {
    return <PortfolioSkeleton />;
  }

  return (
    <div className="h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container pb-16 md:pb-0">
      <div className="flex-1">
        {assets
          ?.sort((a, b) => b.price * b.amount - a.price * a.amount)
          .map((asset) => (
            <div
              key={`${asset.address}-${asset.chain}`}
              className="p-3 sm:p-4 hover:bg-black/50 transition-colors"
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
                      <div
                        className="hover:text-blue-500 truncate max-w-[90px] sm:max-w-none cursor-pointer"
                        onClick={() => openChart(asset.address)}
                      >
                        {asset.name}
                      </div>
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
                    {asset.chain === "solana" &&
                      asset.address ===
                        "So11111111111111111111111111111111111111112" && (
                        <button
                          className="cursor-pointer border border-[#2D2D2D] rounded-full p-2 bg-transparent hover:bg-[#2D2D2D] transition-colors"
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
                {asset.chain === "solana" && (
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleOpenModal(asset, "buy")}
                      className="px-2 py-1 bg-green-500/20 hover:bg-green-500/30 text-green-300 border border-green-500/30 rounded-lg text-xs transition-colors flex items-center gap-1"
                    >
                      <FaShoppingCart size={12} />
                    </button>
                    <button
                      onClick={() => handleOpenModal(asset, "sell")}
                      className="px-2 py-1 bg-red-500/20 hover:bg-red-500/30 text-red-300 border border-red-500/30 rounded-lg text-xs transition-colors flex items-center gap-1"
                    >
                      <IoArrowDown size={12} />
                    </button>
                  </div>
                )}
              </div>
            </div>
          ))}
        {(!assets || assets.length === 0) && (
          <div className="text-center text-gray-400 flex flex-col items-center gap-2">
            {t("portfolio.no_assets_found")}
            <button onClick={handleRefresh}>
              <FaSync size={12} />
            </button>
          </div>
        )}
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

import { usePrivy } from "@privy-io/react-auth";
import { useFundWallet } from "@privy-io/react-auth/solana";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { FaSync } from "react-icons/fa";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";
import { BuySellModal } from "./BuySellModal";
import { PortfolioItemTile } from "./PortfolioItemTile";
import { PortfolioSkeleton } from "./PortfolioSkeleton";

export function Portfolio() {
  const { solanaAddress } = useWalletStore();
  const { user } = usePrivy();
  const {
    getCombinedPortfolio,
    isLoading,
    refreshPortfolio,
    initializePortfolioManager,
  } = usePortfolioStore();

  const { t } = useTranslation();
  const { fundWallet } = useFundWallet();

  const [modalOpen, setModalOpen] = useState(false);
  const [modalAction, setModalAction] = useState<"buy" | "sell">("buy");
  const [selectedAsset, setSelectedAsset] = useState<any>(null);

  // Only run once on mount
  useEffect(() => {
    if (!isLoading) {
      initializePortfolioManager();
    }
  }, []);

  const handleTopup = async () => {
    if (solanaAddress) {
      await fundWallet(solanaAddress);
    }
  };

  const handleOpenModal = (asset: any, action: "buy" | "sell") => {
    setSelectedAsset(asset);
    setModalAction(action);
    setModalOpen(true);
  };

  // Get assets using the selector
  const assets = getCombinedPortfolio();

  if (!user || !solanaAddress) {
    return <></>;
  }

  if (isLoading && (!assets || assets.length === 0)) {
    return <PortfolioSkeleton />;
  }

  return (
    <div className="h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container pb-16 md:pb-0">
      <div className="flex-1">
        {assets
          ?.sort((a, b) => b.price * b.amount - a.price * a.amount)
          .map((asset) => (
            <PortfolioItemTile
              key={`${asset.address}-${asset.chain}`}
              asset={asset}
              onBuy={(asset) => handleOpenModal(asset, "buy")}
              onSell={(asset) => handleOpenModal(asset, "sell")}
              onTopup={handleTopup}
            />
          ))}
        {(!assets || assets.length === 0) && (
          <div className="text-center text-gray-400 flex flex-col items-center gap-2">
            {t("portfolio.no_assets_found")}
            <button onClick={refreshPortfolio}>
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

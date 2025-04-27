import { useState } from "react";
import { useMobile } from "../contexts/MobileContext";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";
import { BuySellModal } from "./BuySellModal";
import { PortfolioItemTile } from "./PortfolioItemTile";
import { PortfolioSkeleton } from "./PortfolioSkeleton";
import { PortfolioSummary } from "./PortfolioSummary";

export function Portfolio() {
  const { getCombinedPortfolio, isLoading } = usePortfolioStore();
  const { solanaAddress, evmAddress } = useWalletStore();

  const [modalOpen, setModalOpen] = useState(false);
  const [modalAction, setModalAction] = useState<"buy" | "sell">("buy");
  const [selectedAsset, setSelectedAsset] = useState<any>(null);
  const { isMobile } = useMobile();

  const hasWallet = Boolean(solanaAddress || evmAddress);

  const handleOpenModal = (asset: any, action: "buy" | "sell") => {
    setSelectedAsset(asset);
    setModalAction(action);
    setModalOpen(true);
  };

  // Get assets using the selector
  const assets = getCombinedPortfolio();

  // Calculate total balance from assets
  const totalBalance =
    assets?.reduce((sum, asset) => sum + asset.price * asset.amount, 0) || 0;

  // Only show loading state if we have a wallet and are actually loading
  if (hasWallet && isLoading) {
    return <PortfolioSkeleton />;
  }

  return (
    <div
      className={`h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container pb-16 md:pb-0 ${
        isMobile ? "p-0" : "p-4"
      }`}
    >
      <PortfolioSummary totalBalance={totalBalance} />
      <div className="flex-1 space-y-2">
        {assets
          ?.sort((a, b) => b.price * b.amount - a.price * a.amount)
          .map((asset) => (
            <PortfolioItemTile
              key={`${asset.address}-${asset.chain}`}
              asset={asset}
              onBuy={(asset) => handleOpenModal(asset, "buy")}
              onSell={(asset) => handleOpenModal(asset, "sell")}
            />
          ))}
      </div>

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

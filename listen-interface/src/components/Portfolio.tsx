import { useState } from "react";
import { useMobile } from "../contexts/MobileContext";
import { useWalletStore } from "../store/walletStore";
import { useSettingsStore } from "../store/settingsStore";
import { useSolanaPortfolio } from "../hooks/useSolanaPortfolio";
import { useEvmPortfolio } from "../hooks/useEvmPortfolio";
import { useHyperliquidPortfolio } from "../hooks/useHyperliquidPortfolio";
import { BuySellModal } from "./BuySellModal";
import { PortfolioItemTile } from "./PortfolioItemTile";
import { PortfolioSkeleton } from "./PortfolioSkeleton";
import { PortfolioSummary } from "./PortfolioSummary";
import { WalletSwitcher } from "./WalletSwitcher";
import { PortfolioItem } from "../lib/types";

export function Portfolio() {
  const { 
    solanaAddress, 
    evmAddress, 
    eoaSolanaAddress, 
    eoaEvmAddress, 
    activeWallet 
  } = useWalletStore();
  
  const { hyperliquid } = useSettingsStore();
  const quickBuyAvailable = activeWallet === "listen";

  const [modalOpen, setModalOpen] = useState(false);
  const [modalAction, setModalAction] = useState<"buy" | "sell">("buy");
  const [selectedAsset, setSelectedAsset] = useState<any>(null);
  const { isMobile } = useMobile();

  // Get addresses based on active wallet
  const currentSolanaAddress = activeWallet === "listen" ? solanaAddress : 
                              activeWallet === "eoaSolana" ? eoaSolanaAddress : null;
  const currentEvmAddress = activeWallet === "listen" ? evmAddress : 
                           activeWallet === "eoaEvm" ? eoaEvmAddress : null;

  // Use individual portfolio hooks
  const solanaQuery = useSolanaPortfolio(currentSolanaAddress);
  const evmQuery = useEvmPortfolio(currentEvmAddress);
  const hyperliquidQuery = useHyperliquidPortfolio(
    hyperliquid && activeWallet === "listen" ? evmAddress : null
  );

  const handleOpenModal = (asset: any, action: "buy" | "sell") => {
    setSelectedAsset(asset);
    setModalAction(action);
    setModalOpen(true);
  };

  // Combine all portfolio data
  const assets: PortfolioItem[] = [
    ...(solanaQuery.data || []),
    ...(evmQuery.data || []),
    ...(hyperliquidQuery.data || []),
  ];

  // Calculate total balance from assets
  const totalBalance = assets.reduce((sum, asset) => sum + asset.price * asset.amount, 0);
  
  // Calculate 24h PnL percentage
  const portfolioPnL = assets.length > 0 ? assets.reduce((weightedPnL, asset) => {
    const assetValue = asset.price * asset.amount;
    return weightedPnL + (asset.priceChange24h * assetValue);
  }, 0) / totalBalance : 0;

  // Check if any query is loading
  const isLoading = solanaQuery.isLoading || evmQuery.isLoading || hyperliquidQuery.isLoading;
  const hasWallet = Boolean(currentSolanaAddress || currentEvmAddress);

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
      <WalletSwitcher />
      <PortfolioSummary totalBalance={totalBalance} portfolioPnL={portfolioPnL} />
      <div className="flex-1 space-y-2">
        {assets
          ?.sort((a, b) => b.price * b.amount - a.price * a.amount)
          .map((asset) => (
            <PortfolioItemTile
              key={`${asset.address}-${asset.chain}`}
              asset={asset}
              onBuy={
                quickBuyAvailable
                  ? (asset) => handleOpenModal(asset, "buy")
                  : undefined
              }
              onSell={
                quickBuyAvailable
                  ? (asset) => handleOpenModal(asset, "sell")
                  : undefined
              }
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

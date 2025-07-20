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
import { PortfolioZeroState } from "./PortfolioZeroState";
import { WalletSwitcher } from "./WalletSwitcher";
import { EoaEvmWalletSelector } from "./EoaEvmWalletSelector";
import { PortfolioItem } from "../lib/types";
import { aggregatePortfolioItems } from "../lib/portfolioHelpers";
import { ensurePortfolioItem } from "../lib/util";

export function Portfolio() {
  const {
    solanaAddress,
    evmAddress,
    eoaSolanaAddress,
    eoaEvmAddress,
    eoaEvmWallets,
    selectedEoaEvmIndex,
    activeWallet,
  } = useWalletStore();

  const { hyperliquid } = useSettingsStore();
  const isListenWallet = activeWallet === "listen";

  const [modalOpen, setModalOpen] = useState(false);
  const [modalAction, setModalAction] = useState<"buy" | "sell">("buy");
  const [selectedAsset, setSelectedAsset] = useState<PortfolioItem | null>(
    null,
  );
  const { isMobile } = useMobile();

  // Get addresses based on active wallet
  const currentSolanaAddress =
    activeWallet === "listen"
      ? solanaAddress
      : activeWallet === "eoaSolana"
        ? eoaSolanaAddress
        : null;

  // For EOA EVM, use the selected wallet from the list
  const selectedEoaEvmWallet = eoaEvmWallets[selectedEoaEvmIndex];
  const currentEvmAddress =
    activeWallet === "listen"
      ? evmAddress
      : activeWallet === "eoaEvm"
        ? selectedEoaEvmWallet?.address || eoaEvmAddress
        : null;

  // Use individual portfolio hooks
  const solanaQuery = useSolanaPortfolio(currentSolanaAddress);
  const evmQuery = useEvmPortfolio(currentEvmAddress);

  // For Hyperliquid, we need to use the appropriate EVM address based on active wallet
  const hyperliquidAddress =
    activeWallet === "listen"
      ? evmAddress
      : activeWallet === "eoaEvm"
        ? selectedEoaEvmWallet?.address || eoaEvmAddress
        : null;

  // Always call the hook, but control with enabled flag
  const hyperliquidQuery = useHyperliquidPortfolio(
    hyperliquidAddress,
    hyperliquid && !!hyperliquidAddress,
  );

  const handleOpenModal = (
    asset: PortfolioItem | null,
    action: "buy" | "sell",
  ) => {
    if (!asset) return;
    setSelectedAsset(asset);
    setModalAction(action);
    setModalOpen(true);
  };

  // Combine all portfolio data
  const rawAssets: PortfolioItem[] = [
    ...(solanaQuery.data || []),
    ...(evmQuery.data || []),
    ...(hyperliquidQuery.data || []),
  ];

  // Aggregate assets with the same symbol across chains
  const aggregatedAssets = aggregatePortfolioItems(rawAssets);

  // Filter out assets with USD value <= $0.02
  const assets = aggregatedAssets.filter(
    (asset) => asset.price * asset.amount > 0.02,
  );

  // Calculate total balance from aggregated assets
  const totalBalance = assets.reduce(
    (sum, asset) => sum + asset.price * asset.amount,
    0,
  );

  // Calculate 24h PnL percentage
  const portfolioPnL =
    assets.length > 0
      ? assets.reduce((weightedPnL, asset) => {
          const assetValue = asset.price * asset.amount;
          return weightedPnL + asset.priceChange24h * assetValue;
        }, 0) / totalBalance
      : 0;

  // Check if any query is loading
  const isLoading =
    solanaQuery.isLoading || evmQuery.isLoading || hyperliquidQuery.isLoading;
  const hasWallet = Boolean(currentSolanaAddress || currentEvmAddress);

  // Only show loading state if we have a wallet and are actually loading
  if (isLoading) {
    return <PortfolioSkeleton />;
  }

  console.log({ isListenWallet, hasWallet, len: assets.length });

  // Show zero state if we have a wallet but no assets
  if (isListenWallet && !isLoading && assets.length === 0) {
    return (
      <div
        className={`h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container pb-16 md:pb-0 ${
          isMobile ? "p-0" : "p-4"
        }`}
      >
        <WalletSwitcher />
        <PortfolioZeroState />
      </div>
    );
  }

  return (
    <div
      className={`h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container pb-16 md:pb-0 ${
        isMobile ? "p-0" : "p-4"
      }`}
    >
      <WalletSwitcher />
      {activeWallet === "eoaEvm" && <EoaEvmWalletSelector />}
      <PortfolioSummary
        totalBalance={totalBalance}
        portfolioPnL={portfolioPnL}
      />
      <div className="flex-1 space-y-2">
        {assets
          ?.sort((a, b) => b.price * b.amount - a.price * a.amount)
          .map((asset) => (
            <PortfolioItemTile
              key={`${asset.address}-${asset?.chains?.join("-")}`}
              asset={asset}
              onBuy={
                isListenWallet
                  ? (asset) =>
                      handleOpenModal(ensurePortfolioItem(asset), "buy")
                  : undefined
              }
              onSell={
                isListenWallet
                  ? (asset) =>
                      handleOpenModal(ensurePortfolioItem(asset), "sell")
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

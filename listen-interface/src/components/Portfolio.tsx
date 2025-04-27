import { useFundWallet, usePrivy } from "@privy-io/react-auth";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { FaSync } from "react-icons/fa";
import { MdOutlineArrowOutward } from "react-icons/md";
import { TbDots, TbPlus } from "react-icons/tb";
import { useMobile } from "../contexts/MobileContext";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";
import { BuySellModal } from "./BuySellModal";
import { PortfolioItemTile } from "./PortfolioItemTile";
import { PortfolioSkeleton } from "./PortfolioSkeleton";
import TileButton from "./TileButton";

interface PortfolioSummaryProps {
  totalBalance: number;
}

export function PortfolioSummary({ totalBalance }: PortfolioSummaryProps) {
  const { solanaAddress } = useWalletStore();
  const { fundWallet } = useFundWallet();

  const handleTopup = async () => {
    if (solanaAddress) {
      await fundWallet(solanaAddress);
    }
  };

  return (
    <div className="flex flex-col justify-center p-10 gap-7 w-full bg-[#151518] bg-opacity-40 border border-white/[0.04] rounded-[20px]">
      <span className="font-space-grotesk font-medium text-[52px] leading-4 text-white text-center">
        $
        {totalBalance.toLocaleString(undefined, {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        })}
      </span>
      <div className="flex flex-row items-center gap-3 justify-center mt-2">
        <TileButton
          icon={<TbPlus className="w-4 h-4" />}
          onClick={handleTopup}
          ariaLabel="Deposit"
        />
        <TileButton
          icon={<MdOutlineArrowOutward />}
          onClick={handleTopup}
          ariaLabel="Withdraw"
        />
        <TileButton
          icon={<TbDots className="w-5 h-5" />}
          onClick={() => {}}
          ariaLabel="Refresh"
        />
      </div>
    </div>
  );
}

export function Portfolio() {
  const { solanaAddress } = useWalletStore();
  const { user } = usePrivy();
  const { getCombinedPortfolio, isLoading, refreshPortfolio } =
    usePortfolioStore();
  const { isMobile } = useMobile();

  const { t } = useTranslation();

  const [modalOpen, setModalOpen] = useState(false);
  const [modalAction, setModalAction] = useState<"buy" | "sell">("buy");
  const [selectedAsset, setSelectedAsset] = useState<any>(null);

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

  if (!user || !solanaAddress) {
    return <></>;
  }

  if (isLoading && (!assets || assets.length === 0)) {
    return <PortfolioSkeleton />;
  }

  return (
    <div className="h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container pb-16 md:pb-0">
      {isMobile && <PortfolioSummary totalBalance={totalBalance} />}
      <div className="flex-1">
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
        {(!assets || assets.length === 0) && !isLoading && (
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

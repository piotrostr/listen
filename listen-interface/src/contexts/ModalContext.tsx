import { createContext, ReactNode, useContext, useState } from "react";
import { createPortal } from "react-dom";
import { MdOutlineArrowOutward } from "react-icons/md";
import { TbPlus } from "react-icons/tb";
import { Chart } from "../components/Chart";
import { GeckoTerminalChart } from "../components/GeckoTerminalChart";
import { ShareModal } from "../components/ShareModal";

interface ChartAsset {
  mint: string;
  chainId?: string;
  onBuy?: () => void;
  onSell?: () => void;
  name?: string;
  symbol?: string;
  amount?: number;
  logoURI?: string | null;
  price?: number;
  decimals?: number;
}

export interface BuySellModalState {
  isOpen: boolean;
  action: "buy" | "sell";
  asset: {
    address: string;
    name: string;
    symbol: string;
    amount: number;
    logoURI?: string;
    price: number;
    decimals: number;
  } | null;
}

interface ModalContextType {
  openChart: (asset: ChartAsset) => void;
  closeChart: () => void;
  openShareModal: (url: string) => void;
  closeShareModal: () => void;
  openBuySellModal: (action: "buy" | "sell", asset: ChartAsset) => void;
  closeBuySellModal: () => void;
  buySellModalState: BuySellModalState;
  returnToChart: () => void;
  hasChartToReturnTo: boolean;
}

const ModalContext = createContext<ModalContextType | null>(null);

export function ModalProvider({ children }: { children: ReactNode }) {
  const [chartAsset, setChartAsset] = useState<ChartAsset | null>(null);
  const [previousChartAsset, setPreviousChartAsset] =
    useState<ChartAsset | null>(null);
  const [isShareModalOpen, setIsShareModalOpen] = useState(false);
  const [shareUrl, setShareUrl] = useState("");
  const [buySellModalState, setBuySellModalState] = useState<BuySellModalState>(
    {
      isOpen: false,
      action: "buy",
      asset: null,
    }
  );

  const openChart = (asset: ChartAsset) => {
    setPreviousChartAsset(null);
    setChartAsset(asset);
  };

  const closeChart = () => setChartAsset(null);

  const openShareModal = (url: string) => {
    setShareUrl(url);
    setIsShareModalOpen(true);
  };

  const closeShareModal = () => {
    setIsShareModalOpen(false);
  };

  const openBuySellModal = (action: "buy" | "sell", asset: ChartAsset) => {
    if (
      !asset.name ||
      !asset.symbol ||
      !asset.amount ||
      !asset.price ||
      !asset.decimals
    ) {
      console.error("Missing required asset properties for BuySellModal");
      return;
    }

    setPreviousChartAsset(chartAsset); // Store the current chart
    setChartAsset(null); // Close chart modal
    setBuySellModalState({
      isOpen: true,
      action,
      asset: {
        address: asset.mint,
        name: asset.name,
        symbol: asset.symbol,
        amount: asset.amount,
        logoURI: asset.logoURI || undefined,
        price: asset.price,
        decimals: asset.decimals,
      },
    });
  };

  const closeBuySellModal = () => {
    setBuySellModalState((prev) => ({ ...prev, isOpen: false }));
  };

  const returnToChart = () => {
    if (previousChartAsset) {
      setBuySellModalState((prev) => ({ ...prev, isOpen: false }));
      setChartAsset(previousChartAsset);
      setPreviousChartAsset(null);
    }
  };

  return (
    <ModalContext.Provider
      value={{
        openChart,
        closeChart,
        openShareModal,
        closeShareModal,
        openBuySellModal,
        closeBuySellModal,
        buySellModalState,
        returnToChart,
        hasChartToReturnTo: !!previousChartAsset,
      }}
    >
      {children}
      {chartAsset &&
        createPortal(
          <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div className="fixed inset-0 bg-[#151518]/60 backdrop-blur-sm pointer-events-none" />
            <div className="relative bg-[#151518]/40 w-[90vw] h-[80vh] rounded-xl p-6 backdrop-blur-sm pointer-events-auto">
              <button
                onClick={closeChart}
                className="absolute top-4 right-4 text-white transition-colors"
              >
                ✕
              </button>
              <div className="flex flex-col h-full">
                {chartAsset.chainId ? (
                  <GeckoTerminalChart
                    tokenAddress={chartAsset.mint}
                    chainId={chartAsset.chainId}
                    timeframe="24h"
                  />
                ) : (
                  <Chart mint={chartAsset.mint} />
                )}
                {chartAsset.onBuy && chartAsset.onSell && (
                  <div className="flex gap-2 justify-center mt-4">
                    <button
                      onClick={() => openBuySellModal("buy", chartAsset)}
                      className="px-2 py-1 bg-green-500/20 hover:bg-green-500/30 text-green-300 border border-green-500/30 rounded-lg text-xs transition-colors flex items-center gap-2"
                    >
                      <TbPlus size={12} />
                      <span>Buy</span>
                    </button>
                    <button
                      onClick={() => openBuySellModal("sell", chartAsset)}
                      className="px-2 py-1 bg-red-500/20 hover:bg-red-500/30 text-red-300 border border-red-500/30 rounded-lg text-xs transition-colors flex items-center gap-2"
                    >
                      <MdOutlineArrowOutward size={12} />
                      <span>Sell</span>
                    </button>
                  </div>
                )}
              </div>
            </div>
            <div className="fixed inset-0 z-[-1]" onClick={closeChart} />
          </div>,
          document.body
        )}
      {isShareModalOpen && (
        <ShareModal url={shareUrl} onClose={closeShareModal} />
      )}
    </ModalContext.Provider>
  );
}

export const useModal = () => {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error("useModal must be used within a ModalProvider");
  }
  return context;
};

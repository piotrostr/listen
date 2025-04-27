import { createContext, ReactNode, useContext, useState } from "react";
import { createPortal } from "react-dom";
import { Chart } from "../components/Chart";
import { GeckoTerminalChart } from "../components/GeckoTerminalChart";
import { ShareModal } from "../components/ShareModal";

interface ModalContextType {
  openChart: (asset: ChartAsset) => void;
  closeChart: () => void;
  openShareModal: (url: string) => void;
  closeShareModal: () => void;
}

const ModalContext = createContext<ModalContextType | null>(null);

interface ChartAsset {
  mint: string;
  chainId?: string;
}

export function ModalProvider({ children }: { children: ReactNode }) {
  const [chartAsset, setChartAsset] = useState<ChartAsset | null>(null);
  const [isShareModalOpen, setIsShareModalOpen] = useState(false);
  const [shareUrl, setShareUrl] = useState("");

  const openChart = (asset: ChartAsset) => setChartAsset(asset);
  const closeChart = () => setChartAsset(null);

  const openShareModal = (url: string) => {
    setShareUrl(url);
    setIsShareModalOpen(true);
  };

  const closeShareModal = () => {
    setIsShareModalOpen(false);
  };

  return (
    <ModalContext.Provider
      value={{
        openChart,
        closeChart,
        openShareModal,
        closeShareModal,
      }}
    >
      {children}
      {chartAsset &&
        createPortal(
          <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div className="fixed inset-0 bg-[#151518]/60 backdrop-blur-sm pointer-events-none" />
            <div className="relative bg-[#151518]/40  w-[90vw] h-[80vh] rounded-xl p-6 backdrop-blur-sm pointer-events-auto">
              <button
                onClick={closeChart}
                className="absolute top-4 right-4 text-white transition-colors"
              >
                ✕
              </button>
              {chartAsset.chainId ? (
                <div className="w-full h-full">
                  <GeckoTerminalChart
                    tokenAddress={chartAsset.mint}
                    chainId={chartAsset.chainId}
                    timeframe="24h"
                  />
                </div>
              ) : (
                <div className="w-full h-full">
                  <Chart mint={chartAsset.mint} />
                </div>
              )}
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

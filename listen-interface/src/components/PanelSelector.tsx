import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoRefreshOutline } from "react-icons/io5";
import { useMobile } from "../contexts/MobileContext";
import { usePortfolio } from "../hooks/usePortfolio";
import { Chat } from "./Chat";
import { FloatingPanel } from "./FloatingPanel";
import { Pipelines, PipelinesHeader } from "./Pipelines";
import { Portfolio } from "./Portfolio";
import { PriceUpdates, PriceUpdatesHeader } from "./PriceUpdates";
import { Settings } from "./Settings";

export function PanelSelector({
  activePanel,
  setActivePanel,
}: {
  activePanel: string | null;
  setActivePanel: (panel: string | null) => void;
}) {
  const [marketCapFilter, setMarketCapFilter] = useState<string>("all");
  const [volumeFilter, setVolumeFilter] = useState<"bought" | "sold" | "all">(
    "all"
  );
  const [isListFrozen, setIsListFrozen] = useState(false);
  const [showWatchlistOnly, setShowWatchlistOnly] = useState(false);
  const [showHiddenOnly, setShowHiddenOnly] = useState(false);
  const [statusFilter, setStatusFilter] = useState<string>("All");
  const { isMobile } = useMobile();
  const queryClient = useQueryClient();

  const handleClose = useCallback(() => {
    setActivePanel(null);
  }, [setActivePanel]);

  const handlePortfolioRefresh = useCallback(async () => {
    await queryClient.resetQueries({
      queryKey: ["portfolio"],
      exact: false,
    });
  }, [queryClient]);

  if (!activePanel) return null;

  // For mobile, render just the contents without the FloatingPanel wrapper
  if (isMobile) {
    if (activePanel === "screener") {
      return (
        <div className="h-full bg-black">
          <div className="mb-4">
            <PriceUpdatesHeader
              volumeFilter={volumeFilter}
              setVolumeFilter={setVolumeFilter}
              marketCapFilter={marketCapFilter}
              setMarketCapFilter={setMarketCapFilter}
              isListFrozen={isListFrozen}
              showWatchlistOnly={showWatchlistOnly}
              setShowWatchlistOnly={setShowWatchlistOnly}
              showHiddenOnly={showHiddenOnly}
              setShowHiddenOnly={setShowHiddenOnly}
            />
          </div>
          <PriceUpdates
            marketCapFilter={marketCapFilter}
            volumeFilter={volumeFilter}
            isListFrozen={isListFrozen}
            setIsListFrozen={setIsListFrozen}
            showWatchlistOnly={showWatchlistOnly}
            showHiddenOnly={showHiddenOnly}
          />
        </div>
      );
    }

    if (activePanel === "pipelines") {
      return (
        <div className="h-full bg-black">
          <div className="mb-4">
            <PipelinesHeader
              statusFilter={statusFilter}
              setStatusFilter={setStatusFilter}
            />
          </div>
          <Pipelines statusFilter={statusFilter} />
        </div>
      );
    }

    if (activePanel === "chat") {
      return <Chat />;
    }

    if (activePanel === "portfolio") {
      return (
        <div className="h-full bg-black">
          <div className="mb-4">
            <PortfolioHeader onRefresh={handlePortfolioRefresh} />
          </div>
          <Portfolio />
        </div>
      );
    }

    if (activePanel === "settings") {
      return (
        <div className="h-full bg-black">
          <Settings />
        </div>
      );
    }

    return null;
  }

  // For desktop, use the FloatingPanel wrapper
  return (
    <div className="h-full pr-4">
      {activePanel === "screener" && (
        <FloatingPanel
          title="screener"
          onClose={handleClose}
          headerContent={
            <PriceUpdatesHeader
              volumeFilter={volumeFilter}
              setVolumeFilter={setVolumeFilter}
              marketCapFilter={marketCapFilter}
              setMarketCapFilter={setMarketCapFilter}
              isListFrozen={isListFrozen}
              showWatchlistOnly={showWatchlistOnly}
              setShowWatchlistOnly={setShowWatchlistOnly}
              showHiddenOnly={showHiddenOnly}
              setShowHiddenOnly={setShowHiddenOnly}
            />
          }
        >
          <PriceUpdates
            marketCapFilter={marketCapFilter}
            volumeFilter={volumeFilter}
            isListFrozen={isListFrozen}
            setIsListFrozen={setIsListFrozen}
            showWatchlistOnly={showWatchlistOnly}
            showHiddenOnly={showHiddenOnly}
          />
        </FloatingPanel>
      )}

      {activePanel === "pipelines" && (
        <FloatingPanel
          title="pipelines"
          onClose={handleClose}
          headerContent={
            <PipelinesHeader
              statusFilter={statusFilter}
              setStatusFilter={setStatusFilter}
            />
          }
        >
          <Pipelines statusFilter={statusFilter} />
        </FloatingPanel>
      )}

      {activePanel === "chat" && (
        <FloatingPanel title="chat" onClose={handleClose}>
          <Chat />
        </FloatingPanel>
      )}

      {activePanel === "portfolio" && (
        <FloatingPanel
          title="portfolio"
          onClose={handleClose}
          headerContent={<PortfolioHeader onRefresh={handlePortfolioRefresh} />}
        >
          <Portfolio />
        </FloatingPanel>
      )}

      {activePanel === "settings" && (
        <FloatingPanel title="settings" onClose={handleClose}>
          <Settings />
        </FloatingPanel>
      )}
    </div>
  );
}

// New Portfolio Header component
function PortfolioHeader({ onRefresh }: { onRefresh: () => Promise<void> }) {
  const { t } = useTranslation();
  const { portfolioValue, isLoading } = usePortfolio();

  return (
    <div className="flex items-center justify-between w-full">
      <div className="flex items-center gap-2">
        <span className="font-mono text-sm">
          {isLoading ? "..." : `$${portfolioValue.toFixed(2)}`}
        </span>
      </div>
      <div className="flex items-center gap-2">
        <button
          onClick={onRefresh}
          className="bg-black/40 text-white border border-[#2D2D2D] rounded-lg w-8 h-8 flex items-center justify-center hover:bg-white/10"
          title={t("portfolio.refresh")}
        >
          <IoRefreshOutline className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}

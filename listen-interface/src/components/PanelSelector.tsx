import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { IoCloseOutline, IoRefreshOutline } from "react-icons/io5";
import { useMobile } from "../contexts/MobileContext";
import { usePortfolioStore } from "../store/portfolioStore";
import { Chat } from "./Chat";
import { FloatingPanel } from "./FloatingPanel";
import { Pipelines, PipelinesHeader } from "./Pipelines";
import { Portfolio } from "./Portfolio";
import { PriceUpdates } from "./PriceUpdates";
import { PriceUpdatesHeader } from "./PriceUpdatesHeader";
import { Settings } from "./Settings";

export function PanelSelector({
  activePanel,
  setActivePanel,
}: {
  activePanel: string | null;
  setActivePanel: (panel: string | null) => void;
}) {
  const [statusFilter, setStatusFilter] = useState<string>("All");
  const { isMobile } = useMobile();
  const { t } = useTranslation();

  const { refreshPortfolio } = usePortfolioStore();

  const handleClose = useCallback(() => {
    setActivePanel(null);
  }, [setActivePanel]);

  if (!activePanel) return null;

  // For mobile, render just the contents without the FloatingPanel wrapper
  if (isMobile) {
    // Mobile header with close button
    const MobileHeader = ({ children }: { children: React.ReactNode }) => (
      <div className="flex items-center justify-between mb-4">
        <div className="flex-1">{children}</div>
        <button
          onClick={handleClose}
          className="bg-black/40 text-white border border-[#2D2D2D] rounded-lg w-8 h-8 flex items-center justify-center hover:bg-white/10 ml-2"
          title={t("common.close")}
        >
          <IoCloseOutline className="w-5 h-5" />
        </button>
      </div>
    );

    if (activePanel === "screener") {
      return (
        <div className="h-full bg-black">
          <MobileHeader>
            <PriceUpdatesHeader />
          </MobileHeader>
          <PriceUpdates />
        </div>
      );
    }

    if (activePanel === "pipelines") {
      return (
        <div className="h-full bg-black">
          <MobileHeader>
            <PipelinesHeader
              statusFilter={statusFilter}
              setStatusFilter={setStatusFilter}
            />
          </MobileHeader>
          <Pipelines statusFilter={statusFilter} />
        </div>
      );
    }

    if (activePanel === "chat") {
      return (
        <div className="h-full bg-black">
          <MobileHeader>
            <div className="text-white font-medium">{t("layout.chat")}</div>
          </MobileHeader>
          <Chat />
        </div>
      );
    }

    if (activePanel === "portfolio") {
      return (
        <div className="h-full bg-black">
          <MobileHeader>
            <PortfolioHeader onRefresh={refreshPortfolio} />
          </MobileHeader>
          <Portfolio />
        </div>
      );
    }

    if (activePanel === "settings") {
      return (
        <div className="h-full bg-black">
          <MobileHeader>
            <div className="text-white font-medium">{t("layout.settings")}</div>
          </MobileHeader>
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
          headerContent={<PriceUpdatesHeader />}
        >
          <PriceUpdates />
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
          headerContent={<PortfolioHeader onRefresh={refreshPortfolio} />}
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
  const { getPortfolioValue, isLoading } = usePortfolioStore();
  const portfolioValue = getPortfolioValue();

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

import { useCallback, useEffect, useState } from "react";
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
  // Price Updates state
  const [marketCapFilter, setMarketCapFilter] = useState<string>("all");
  const [volumeFilter, setVolumeFilter] = useState<"bought" | "sold" | "all">(
    "all"
  );
  const [isListFrozen, setIsListFrozen] = useState(false);

  // Pipelines state
  const [statusFilter, setStatusFilter] = useState<string>("All");

  // Check if we're on mobile
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 600);
    };

    checkMobile();
    window.addEventListener("resize", checkMobile);
    return () => window.removeEventListener("resize", checkMobile);
  }, []);

  const handleClose = useCallback(() => {
    setActivePanel(null);
  }, [setActivePanel]);

  if (!activePanel) return null;

  // For mobile, render just the contents without the FloatingPanel wrapper
  if (isMobile) {
    if (activePanel === "screener") {
      return (
        <div className="h-full">
          <div className="mb-4">
            <PriceUpdatesHeader
              volumeFilter={volumeFilter}
              setVolumeFilter={setVolumeFilter}
              marketCapFilter={marketCapFilter}
              setMarketCapFilter={setMarketCapFilter}
              isListFrozen={isListFrozen}
            />
          </div>
          <PriceUpdates
            marketCapFilter={marketCapFilter}
            volumeFilter={volumeFilter}
            isListFrozen={isListFrozen}
            setIsListFrozen={setIsListFrozen}
          />
        </div>
      );
    }

    if (activePanel === "pipelines") {
      return (
        <div className="h-full">
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
      return <Portfolio />;
    }

    if (activePanel === "settings") {
      return <Settings />;
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
            />
          }
        >
          <PriceUpdates
            marketCapFilter={marketCapFilter}
            volumeFilter={volumeFilter}
            isListFrozen={isListFrozen}
            setIsListFrozen={setIsListFrozen}
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
        <FloatingPanel title="portfolio" onClose={handleClose}>
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

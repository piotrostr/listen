import { useCallback, useState } from "react";
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

  const handleClose = useCallback(() => {
    setActivePanel(null);
  }, [setActivePanel]);

  if (!activePanel) return null;

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

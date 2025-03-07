import { FloatingPanel } from "./FloatingPanel";
import { Pipelines } from "./Pipelines";
import { Portfolio } from "./Portfolio";
import { PriceUpdates } from "./PriceUpdates";
import { Settings } from "./Settings";

export function PanelSelector({
  activePanel,
  setActivePanel,
}: {
  activePanel: any;
  setActivePanel: (panel: any) => void;
}) {
  if (!activePanel) return null;

  return (
    <FloatingPanel title={activePanel} onClose={() => setActivePanel(null)}>
      {activePanel === "portfolio" && <Portfolio />}
      {activePanel === "screener" && <PriceUpdates />}
      {activePanel === "pipelines" && <Pipelines />}
      {activePanel === "settings" && <Settings />}
    </FloatingPanel>
  );
}

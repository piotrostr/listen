import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();

  return (
    <div
      className={`w-96 h-full border-l border-purple-500/30 bg-black/40 backdrop-blur-sm shadow-lg transition-all duration-300`}
    >
      {activePanel && (
        <div className="flex flex-col h-full">
          <div className="border-b border-purple-500/30 p-3 flex justify-between items-center">
            <div>{t(`layout.${activePanel}`)}</div>
            <button
              onClick={() => setActivePanel(null)}
              className="p-1 rounded-full hover:bg-purple-500/20"
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>

          <div className="flex-1 overflow-auto">
            {activePanel === "portfolio" && <Portfolio />}
            {activePanel === "screener" && <PriceUpdates />}
            {activePanel === "pipelines" && <Pipelines />}
            {activePanel === "settings" && <Settings />}
          </div>
        </div>
      )}
    </div>
  );
}

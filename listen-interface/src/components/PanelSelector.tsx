import { useTranslation } from "react-i18next";
import { BsLink } from "react-icons/bs";
import { IoSettingsOutline, IoWalletOutline } from "react-icons/io5";
import { RxDashboard } from "react-icons/rx";
import { MobileNavigation } from "./MobileNavigation";
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
    <>
      <div className="relative h-full">
        {/* Panel toggle buttons - desktop only */}
        <div className="absolute top-4 right-4 hidden lg:flex gap-2 z-10">
          <button
            onClick={() => setActivePanel("portfolio")}
            className={`p-2 rounded-lg ${activePanel === "portfolio" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
            title={t("layout.portfolio")}
          >
            <IoWalletOutline className="w-5 h-5" />
          </button>
          <button
            onClick={() => setActivePanel("screener")}
            className={`p-2 rounded-lg ${activePanel === "screener" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
            title={t("layout.screener")}
          >
            <RxDashboard className="w-5 h-5" />
          </button>
          <button
            onClick={() => setActivePanel("pipelines")}
            className={`p-2 rounded-lg ${activePanel === "pipelines" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
            title={t("layout.pipelines")}
          >
            <BsLink className="w-5 h-5" />
          </button>
          <button
            onClick={() => setActivePanel("settings")}
            className={`p-2 rounded-lg ${activePanel === "settings" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
            title={t("layout.settings")}
          >
            <IoSettingsOutline className="w-5 h-5" />
          </button>
        </div>

        {/* Panel content */}
        <div
          className={`lg:w-96 w-full h-full border-l border-purple-500/30 bg-black/40 backdrop-blur-sm transition-all duration-300 scrollable-container ${
            activePanel
              ? "translate-x-0"
              : "lg:translate-x-full translate-y-full"
          } mt-10`}
        >
          {activePanel === "portfolio" && <Portfolio />}
          {activePanel === "screener" && <PriceUpdates />}
          {activePanel === "pipelines" && <Pipelines />}
          {activePanel === "settings" && <Settings />}
        </div>
      </div>

      {/* Mobile Navigation */}
      <MobileNavigation
        activePanel={activePanel}
        setActivePanel={setActivePanel}
      />
    </>
  );
}

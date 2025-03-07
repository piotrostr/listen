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
        <div className="flex-row justify-between items-center border-l border-purple-500/30 bg-black/40 backdrop-blur-sm p-3 lg:flex hidden">
          <div>{t(`layout.${activePanel}`)}</div>
          <div className="flex flex-row gap-2">
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
        </div>

        {/* Panel content */}
        <div
          className={`lg:w-96 w-full h-full border-l border-purple-500/30 bg-black/40 backdrop-blur-sm transition-all duration-300 scrollable-container ${
            activePanel
              ? "translate-x-0"
              : "lg:translate-x-full translate-y-full"
          }`}
        >
          {activePanel === "portfolio" && <Portfolio />}
          {activePanel === "screener" && <PriceUpdates />}
          {activePanel === "pipelines" && <Pipelines />}
          {activePanel === "settings" && <Settings />}
        </div>
      </div>

      <MobileNavigation
        activePanel={activePanel}
        setActivePanel={setActivePanel}
      />
    </>
  );
}

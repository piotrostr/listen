import { useTranslation } from "react-i18next";
import { BsLink } from "react-icons/bs";
import { IoSettingsOutline, IoWalletOutline } from "react-icons/io5";
import { RxDashboard } from "react-icons/rx";

interface SimpleHeaderProps {
  activePanel: string | null;
  setActivePanel: (panel: string | null) => void;
}

export function SimpleHeader({
  activePanel,
  setActivePanel,
}: SimpleHeaderProps) {
  const { t } = useTranslation();

  const togglePanel = (panelName: string) => {
    setActivePanel(activePanel === panelName ? null : panelName);
  };

  return (
    <div className="flex items-center justify-between h-16 px-4">
      {/* Left side - Logo */}
      <div className="flex items-center space-x-3">
        <img src="/listen-more.png" alt="Logo" className="w-8 h-8 rounded" />
        <span className="font-bold text-md lg:text-xl">listen-rs</span>
      </div>

      {/* Right side - Panel toggles */}
      <div className="flex items-center space-x-2">
        <button
          onClick={() => togglePanel("portfolio")}
          className={`p-2 rounded-lg ${activePanel === "portfolio" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
          title={t("layout.portfolio")}
        >
          <IoWalletOutline className="w-5 h-5" />
        </button>
        <button
          onClick={() => togglePanel("screener")}
          className={`p-2 rounded-lg ${activePanel === "screener" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
          title={t("layout.screener")}
        >
          <RxDashboard className="w-5 h-5" />
        </button>
        <button
          onClick={() => togglePanel("pipelines")}
          className={`p-2 rounded-lg ${activePanel === "pipelines" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
          title={t("layout.pipelines")}
        >
          <BsLink className="w-5 h-5" />
        </button>
        <button
          onClick={() => togglePanel("settings")}
          className={`p-2 rounded-lg ${activePanel === "settings" ? "bg-purple-500/40" : "bg-black/40"} hover:bg-purple-500/20 transition-colors`}
          title={t("layout.settings")}
        >
          <IoSettingsOutline className="w-5 h-5" />
        </button>
      </div>
    </div>
  );
}

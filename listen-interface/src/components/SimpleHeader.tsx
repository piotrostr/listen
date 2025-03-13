import { Link } from "@tanstack/react-router";
import { useContext } from "react";
import { useTranslation } from "react-i18next";
import { BsLink } from "react-icons/bs";
import { IoMenu, IoSettingsOutline, IoWalletOutline } from "react-icons/io5";
import { RxDashboard } from "react-icons/rx";
import { TbMoneybag } from "react-icons/tb";
import { useMobile } from "../contexts/MobileContext";
import { usePortfolioStore } from "../store/portfolioStore";
import { SidebarContext } from "./Layout";

interface SimpleHeaderProps {
  activePanel: string | null;
  setActivePanel: (panel: string | null) => void;
  toggleMobileSidebar: () => void;
}

export function SimpleHeader({
  activePanel,
  setActivePanel,
  toggleMobileSidebar,
}: SimpleHeaderProps) {
  const { t } = useTranslation();
  const setSidebarOpen = useContext(SidebarContext);
  const { isMobile } = useMobile();

  const togglePanel = (panelName: string) => {
    setActivePanel(activePanel === panelName ? null : panelName);
  };

  const { portfolioValue } = usePortfolioStore();

  const panelButtonStyle = (active: boolean) =>
    `p-2 rounded-lg ${active ? "bg-[#2D2D2D]" : "bg-black/40"} hover:bg-[#2D2D2D] transition-colors`;

  return (
    <div className="flex items-center justify-between h-16 sm:px-4">
      {/* Left side - Logo with hover effect */}
      <div className="flex items-center">
        {isMobile && (
          <button
            onClick={toggleMobileSidebar}
            className="p-4 text-white focus:outline-none"
          >
            <IoMenu size={24} />
          </button>
        )}
        <Link
          to="/"
          search={{
            new: true,
          }}
          className="flex items-center space-x-3 cursor-pointer"
          onMouseEnter={() => setSidebarOpen(true)}
          onMouseLeave={() => setSidebarOpen(false)}
        >
          <img src="/listen-new.svg" alt="Logo" className="w-8 h-8 rounded" />
          {!isMobile && (
            <span className="text-white text-lg font-bold">Listen</span>
          )}
        </Link>
      </div>

      {/* Right side - Panel toggles */}
      <div className="flex items-center space-x-2">
        {!isMobile && (
          <>
            <button
              onClick={() => togglePanel("portfolio")}
              className={panelButtonStyle(activePanel === "portfolio")}
              title={t("layout.portfolio")}
            >
              <IoWalletOutline className="w-5 h-5" />
            </button>
            <button
              onClick={() => togglePanel("screener")}
              className={panelButtonStyle(activePanel === "screener")}
              title={t("layout.screener")}
            >
              <RxDashboard className="w-5 h-5" />
            </button>
            <button
              onClick={() => togglePanel("pipelines")}
              className={panelButtonStyle(activePanel === "pipelines")}
              title={t("layout.pipelines")}
            >
              <BsLink className="w-5 h-5" />
            </button>
            <button
              onClick={() => togglePanel("settings")}
              className={panelButtonStyle(activePanel === "settings")}
              title={t("layout.settings")}
            >
              <IoSettingsOutline className="w-5 h-5" />
            </button>
          </>
        )}
      </div>
      {isMobile && portfolioValue > 0 && (
        <div
          className="text-white text-sm pr-5 flex items-center gap-1"
          onClick={() => setActivePanel("portfolio")}
        >
          <TbMoneybag className="w-4 h-4" />${portfolioValue.toFixed(2)}
        </div>
      )}
    </div>
  );
}

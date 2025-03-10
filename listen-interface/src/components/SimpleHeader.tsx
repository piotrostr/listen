import { Link } from "@tanstack/react-router";
import { useContext } from "react";
import { useTranslation } from "react-i18next";
import { BsLink } from "react-icons/bs";
import { IoMenu, IoSettingsOutline, IoWalletOutline } from "react-icons/io5";
import { RxDashboard } from "react-icons/rx";
import { useMobile } from "../contexts/MobileContext";
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
          <img src="/listen-more.png" alt="Logo" className="w-8 h-8 rounded" />
          {!isMobile && (
            <span className="font-bold text-md lg:text-xl">Listen</span>
          )}
        </Link>
      </div>

      {/* Right side - Panel toggles */}
      <div className="flex items-center space-x-2">
        {!isMobile && (
          <>
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
          </>
        )}
      </div>
    </div>
  );
}

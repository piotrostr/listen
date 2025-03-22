import { usePrivy } from "@privy-io/react-auth";
import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { BsLink } from "react-icons/bs";
import { IoSettingsOutline, IoWalletOutline } from "react-icons/io5";
import { RxDashboard } from "react-icons/rx";
import { useMobile } from "../contexts/MobileContext";
import { useSidebar } from "../contexts/SidebarContext";
import { usePortfolioStore } from "../store/portfolioStore";
import { BurgerIcon } from "./Burger";

interface SimpleHeaderProps {
  activePanel: string | null;
  setActivePanel: (panel: string | null) => void;
  toggleMobileSidebar: () => void;
}

const WalletIcon = () => {
  const { isVerySmallScreen } = useMobile();

  return (
    <svg
      width={isVerySmallScreen ? "18" : "20"}
      height={isVerySmallScreen ? "16" : "18"}
      viewBox="0 0 20 18"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M18 4V2C18 0.897 17.103 0 16 0H3C1.346 0 0 1.346 0 3V15C0 17.201 1.794 18 3 18H18C19.103 18 20 17.103 20 16V6C20 4.897 19.103 4 18 4ZM16 13H14V9H16V13ZM3 4C2.74252 3.98848 2.49941 3.87809 2.32128 3.69182C2.14315 3.50554 2.04373 3.25774 2.04373 3C2.04373 2.74226 2.14315 2.49446 2.32128 2.30818C2.49941 2.12191 2.74252 2.01152 3 2H16V4H3Z"
        fill="#8DFC63"
      />
    </svg>
  );
};

export function SimpleHeader({
  activePanel,
  setActivePanel,
  toggleMobileSidebar,
}: SimpleHeaderProps) {
  const { t } = useTranslation();
  const { isSidebarOpen, setIsSidebarOpen } = useSidebar();
  const { isMobile, isVerySmallScreen } = useMobile();
  const { user } = usePrivy();

  const togglePanel = (panelName: string) => {
    setActivePanel(activePanel === panelName ? null : panelName);
  };

  const { getPortfolioValue } = usePortfolioStore();
  const portfolioValue = getPortfolioValue();

  const panelButtonStyle = (active: boolean) =>
    `p-2 rounded-lg ${active ? "bg-[#2D2D2D]" : "bg-black/40"} hover:bg-[#2D2D2D] transition-colors`;

  return (
    <>
      {isMobile ? (
        <div
          className={`flex justify-between items-center w-full ${isVerySmallScreen ? "p-[12px]" : "p-[16px]"} ${isVerySmallScreen ? "mt-1" : "mt-2"}`}
        >
          <BurgerIcon isOpen={isSidebarOpen} onClick={toggleMobileSidebar} />
          <div
            className={`text-white ${isVerySmallScreen ? "text-base" : "text-lg"} flex items-center ${isVerySmallScreen ? "gap-2" : "gap-3"}`}
            onClick={user ? () => setActivePanel("portfolio") : () => {}}
          >
            <WalletIcon />${portfolioValue.toFixed(2)}
          </div>
        </div>
      ) : (
        <div className="flex items-center justify-between h-16 sm:px-4">
          <div className="flex items-center">
            <Link
              to="/"
              search={{
                new: true,
              }}
              className="flex items-center space-x-3 cursor-pointer"
              onMouseEnter={() => setIsSidebarOpen(true)}
              onMouseLeave={() => setIsSidebarOpen(false)}
            >
              <img
                src="/listen-new.svg"
                alt="Logo"
                className="w-8 h-8 rounded"
              />
              {!isMobile && (
                <span className="text-white text-lg font-bold">Listen</span>
              )}
            </Link>
          </div>

          {/* Right side - Panel toggles */}
          <div className="flex items-center space-x-2">
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
          </div>
        </div>
      )}
    </>
  );
}

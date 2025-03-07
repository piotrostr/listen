import { usePrivy } from "@privy-io/react-auth";
import { useTranslation } from "react-i18next";
import { BsLink } from "react-icons/bs";
import {
  IoChatboxOutline,
  IoSettingsOutline,
  IoWalletOutline,
} from "react-icons/io5";
import { RxDashboard } from "react-icons/rx";
import { useMobile } from "../contexts/MobileContext";

type NavType = "chat" | "portfolio" | "screener" | "pipelines" | "settings";

interface MobileNavigationProps {
  activePanel: string | null;
  setActivePanel: (panel: any) => void;
}

export function MobileNavigation({
  activePanel,
  setActivePanel,
}: MobileNavigationProps) {
  const { t } = useTranslation();
  const { user } = usePrivy();
  const { isIOS } = useMobile();

  const handleNavClick = (navType: NavType) => {
    if (navType === "chat") {
      // Always show chat, hide any panel
      setActivePanel(null);
    } else {
      // For other nav items, toggle the corresponding panel
      setActivePanel(activePanel === navType ? null : navType);
    }
  };

  if (!user) {
    return null;
  }

  return (
    <div
      className={`md:hidden fixed left-0 right-0 bg-black/60 backdrop-blur-sm border-t border-purple-500/30 z-50 ${
        isIOS ? "bottom-4" : "bottom-0"
      }`}
    >
      <div className="flex justify-around items-center h-16 mb-2">
        <button
          onClick={() => handleNavClick("chat")}
          className={`flex flex-col items-center justify-center p-2 ${activePanel === null ? "text-purple-400" : "text-gray-400"}`}
        >
          <IoChatboxOutline className="w-6 h-6" />
          <span className="text-xs mt-1">{t("layout.chat")}</span>
        </button>

        <button
          onClick={() => handleNavClick("portfolio")}
          className={`flex flex-col items-center justify-center p-2 ${activePanel === "portfolio" ? "text-purple-400" : "text-gray-400"}`}
        >
          <IoWalletOutline className="w-6 h-6" />
          <span className="text-xs mt-1">{t("layout.portfolio")}</span>
        </button>

        <button
          onClick={() => handleNavClick("screener")}
          className={`flex flex-col items-center justify-center p-2 ${activePanel === "screener" ? "text-purple-400" : "text-gray-400"}`}
        >
          <RxDashboard className="w-6 h-6" />
          <span className="text-xs mt-1">{t("layout.screener")}</span>
        </button>

        <button
          onClick={() => handleNavClick("pipelines")}
          className={`flex flex-col items-center justify-center p-2 ${activePanel === "pipelines" ? "text-purple-400" : "text-gray-400"}`}
        >
          <BsLink className="w-6 h-6" />
          <span className="text-xs mt-1">{t("layout.pipelines")}</span>
        </button>

        <button
          onClick={() => handleNavClick("settings")}
          className={`flex flex-col items-center justify-center p-2 ${activePanel === "settings" ? "text-purple-400" : "text-gray-400"}`}
        >
          <IoSettingsOutline className="w-6 h-6" />
          <span className="text-xs mt-1">{t("layout.settings")}</span>
        </button>
      </div>
      {isIOS && <div className="h-4 w-full bg-black/60"></div>}
    </div>
  );
}

import { usePrivy } from "@privy-io/react-auth";
import { Link } from "@tanstack/react-router";
import { memo, useEffect, useState } from "react";
import { Background } from "./Background";

import { useTranslation } from "react-i18next";
import { BsLink } from "react-icons/bs";
import { FaXTwitter } from "react-icons/fa6";
import {
  IoChatboxOutline,
  IoSettingsOutline,
  IoWalletOutline,
} from "react-icons/io5";
import { RxDashboard } from "react-icons/rx";
import { useMobile } from "../contexts/MobileContext";
import { useSidebar } from "../contexts/SidebarContext";
import { PanelSelector } from "./PanelSelector";
import { RecentChats } from "./RecentChats";
import { SimpleHeader } from "./SimpleHeader";
import { SwipeHandler } from "./SwipeHandler";
import { VersionAndLanguageDisplay } from "./VersionAndLanguage";
import { VersionInitializer } from "./VersionInitializer";
import { WalletInitializer } from "./WalletInitializer";
import { WebsocketInitializer } from "./WebsocketInitializer";

// Memoize the BottomLink component
const MemoizedBottomLink = memo(function BottomLink({
  href,
  icon: Icon,
  label,
  isSidebarOpen = true,
}: {
  href: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  isSidebarOpen?: boolean;
}) {
  return (
    <a
      href={href}
      className="flex items-center h-10 rounded-lg text-gray-300 hover:text-white hover:bg-[#212121] transition-colors"
      target="_blank"
      rel="noopener noreferrer"
    >
      <div
        className={`flex items-center h-full ${
          isSidebarOpen ? "px-4 w-full" : "justify-center w-16"
        }`}
      >
        <Icon />
        {isSidebarOpen && <span className="ml-3">{label}</span>}
      </div>
    </a>
  );
});

function getBottomItems(t: (key: string) => string) {
  return [
    // TODO update docs when ready
    // {
    //   href: "https://docs.listen-rs.com",
    //   icon: () => (
    //     <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
    //       <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z" />
    //     </svg>
    //   ),
    //   label: t("layout.documentation"),
    // },
    {
      href: "https://github.com/piotrostr/listen",
      icon: () => (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
        </svg>
      ),
      label: t("layout.github"),
    },
    {
      href: "https://x.com/listenonsol",
      icon: () => <FaXTwitter />,
      label: t("layout.twitter"),
    },
  ] as const;
}

export function Layout({ children }: { children: React.ReactNode }) {
  const { isMobile, isIOS } = useMobile();
  const [activePanel, setActivePanel] = useState(
    isMobile ? null : localStorage.getItem("activePanel") || null
  );
  const { user, logout } = usePrivy();
  const { t } = useTranslation();

  const { isSidebarOpen, setIsSidebarOpen, toggleSidebar, isDropdownOpen } =
    useSidebar();

  // Add useEffect to handle iOS viewport height
  useEffect(() => {
    // Function to set the correct viewport height
    const setViewportHeight = () => {
      // First we get the viewport height and multiply it by 1% to get a value for a vh unit
      const vh = window.innerHeight * 0.01;
      // Then we set the value in the --vh custom property to the root of the document
      document.documentElement.style.setProperty("--vh", `${vh}px`);
    };

    // Set the height initially
    setViewportHeight();

    // Add event listener to reset height on resize/orientation change
    window.addEventListener("resize", setViewportHeight);
    window.addEventListener("orientationchange", setViewportHeight);

    // Clean up
    return () => {
      window.removeEventListener("resize", setViewportHeight);
      window.removeEventListener("orientationchange", setViewportHeight);
    };
  }, []);

  // Call the function with the current translation function
  const BOTTOM_ITEMS = getBottomItems(t);

  // Memoize the bottom items to prevent unnecessary re-renders
  const memoizedBottomItems = BOTTOM_ITEMS.map((item, index) => (
    <MemoizedBottomLink
      key={index}
      href={item.href}
      icon={item.icon}
      label={item.label}
      isSidebarOpen={isSidebarOpen}
    />
  ));

  // Handle sidebar hover effects - only for desktop
  const handleSidebarMouseEnter = () => {
    if (!isMobile) setIsSidebarOpen(true);
  };

  const handleSidebarMouseLeave = () => {
    if (!isMobile && !isDropdownOpen) {
      setIsSidebarOpen(false);
    }
  };

  // Handle burger menu click for mobile
  const toggleMobileSidebar = () => {
    toggleSidebar();
  };

  // Function to handle navigation item clicks
  const handleNavClick = (navType: string | null) => {
    if (navType === "chat") {
      // Always show chat, hide any panel
      setActivePanel(null);
    } else {
      // For other nav items, toggle the corresponding panel
      setActivePanel(activePanel === navType ? null : navType);
    }

    // Close sidebar on mobile after selection
    if (isMobile) {
      setIsSidebarOpen(false);
    }
  };

  return (
    <>
      <WalletInitializer />
      <WebsocketInitializer />
      <VersionInitializer />
      <div
        className="relative flex flex-col text-white overflow-hidden"
        style={{ height: "100dvh" }}
      >
        <Background />

        {/* Header */}
        <div className="z-20 bg-black/10 backdrop-blur-sm flex items-center">
          <div className="flex-1">
            <SimpleHeader
              activePanel={activePanel}
              toggleMobileSidebar={toggleMobileSidebar}
              setActivePanel={(panel) => {
                setActivePanel(panel);
                if (panel) {
                  localStorage.setItem("activePanel", panel);
                } else {
                  localStorage.removeItem("activePanel");
                }
              }}
            />
          </div>
        </div>

        {/* Main Content with Sidebar */}
        <div className="flex-1 relative overflow-hidden">
          {/* Add SwipeHandler for mobile */}
          {isMobile && (
            <SwipeHandler
              onOpenSidebar={() => setIsSidebarOpen(true)}
              isOpen={isSidebarOpen}
            />
          )}

          {/* Collapsible Sidebar */}
          <div
            className={`fixed left-0 top-16 bottom-0 z-40 transition-all duration-300 
              ${isMobile ? "w-64" : isSidebarOpen ? "w-64" : "w-16"} 
              ${isMobile ? "transform" : ""} 
              ${isMobile && !isSidebarOpen ? "-translate-x-full" : "translate-x-0"}
              bg-black/90 backdrop-blur-sm flex flex-col ${isMobile ? "pb-5" : ""}`}
            onMouseEnter={handleSidebarMouseEnter}
            onMouseLeave={handleSidebarMouseLeave}
          >
            {/* Content only shown when sidebar is open */}
            {(isSidebarOpen || !isMobile) && (
              <>
                {/* New Chat Button - Only shown on desktop */}
                {isSidebarOpen && !isMobile && (
                  <div className="p-4 flex-shrink-0">
                    <Link
                      to="/"
                      search={{ new: true }}
                      className="flex items-center justify-center h-10 rounded-lg bg-[#2D2D2D] text-white hover:bg-[#2D2D2D]"
                      onClick={() => isMobile && setIsSidebarOpen(false)}
                    >
                      <div className="flex items-center">
                        <svg
                          width="20"
                          height="20"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        >
                          <line x1="12" y1="5" x2="12" y2="19"></line>
                          <line x1="5" y1="12" x2="19" y2="12"></line>
                        </svg>
                        <span className="ml-3">{t("layout.new_chat")}</span>
                      </div>
                    </Link>
                  </div>
                )}

                {/* Mobile Navigation Items - Only shown on mobile */}
                {isMobile && user && isSidebarOpen && (
                  <div className="px-4 mb-4 mt-5 flex-shrink-0">
                    <div className="space-y-1">
                      <button
                        onClick={() => handleNavClick("chat")}
                        className={`flex items-center h-10 w-full rounded-lg ${activePanel === null ? "text-white bg-[#212121]" : "text-gray-300 hover:text-white hover:bg-[#212121]"} transition-colors px-4`}
                      >
                        <IoChatboxOutline className="w-5 h-5" />
                        <span className="ml-3">{t("layout.chat")}</span>
                      </button>

                      <button
                        onClick={() => handleNavClick("portfolio")}
                        className={`flex items-center h-10 w-full rounded-lg ${activePanel === "portfolio" ? "text-white bg-[#212121]" : "text-gray-300 hover:text-white hover:bg-[#212121]"} transition-colors px-4`}
                      >
                        <IoWalletOutline className="w-5 h-5" />
                        <span className="ml-3">{t("layout.portfolio")}</span>
                      </button>

                      <button
                        onClick={() => handleNavClick("screener")}
                        className={`flex items-center h-10 w-full rounded-lg ${activePanel === "screener" ? "text-white bg-[#212121]" : "text-gray-300 hover:text-white hover:bg-[#212121]"} transition-colors px-4`}
                      >
                        <RxDashboard className="w-5 h-5" />
                        <span className="ml-3">{t("layout.screener")}</span>
                      </button>

                      <button
                        onClick={() => handleNavClick("pipelines")}
                        className={`flex items-center h-10 w-full rounded-lg ${activePanel === "pipelines" ? "text-white bg-[#212121]" : "text-gray-300 hover:text-white hover:bg-[#212121]"} transition-colors px-4`}
                      >
                        <BsLink className="w-5 h-5" />
                        <span className="ml-3">{t("layout.pipelines")}</span>
                      </button>

                      <button
                        onClick={() => handleNavClick("settings")}
                        className={`flex items-center h-10 w-full rounded-lg ${activePanel === "settings" ? "text-white bg-[#212121]" : "text-gray-300 hover:text-white hover:bg-[#212121]"} transition-colors px-4`}
                      >
                        <IoSettingsOutline className="w-5 h-5" />
                        <span className="ml-3">{t("layout.settings")}</span>
                      </button>
                    </div>
                  </div>
                )}

                {/* Recent Chats Section - Modified to use flex-grow */}
                <div className="px-4 flex-grow flex flex-col min-h-0">
                  <div
                    className={`${isSidebarOpen ? "block" : "hidden"} h-full`}
                  >
                    <RecentChats
                      onItemClick={() => isMobile && setIsSidebarOpen(false)}
                    />
                  </div>
                </div>

                {/* Bottom section */}
                <div className="mt-auto p-4 space-y-1 flex-shrink-0">
                  {isSidebarOpen && <VersionAndLanguageDisplay />}
                  {memoizedBottomItems}
                  {user && (
                    <button
                      onClick={() => logout()}
                      className="flex items-center h-10 w-full rounded-lg text-gray-300 hover:text-white hover:bg-[#212121] transition-colors"
                    >
                      <div
                        className={`flex items-center h-full ${
                          isSidebarOpen ? "px-4 w-full" : "justify-center w-16"
                        }`}
                      >
                        <svg
                          width="20"
                          height="20"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        >
                          <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4" />
                          <path d="M16 17l5-5-5-5" />
                          <path d="M21 12H9" />
                        </svg>
                        {isSidebarOpen && (
                          <span className="ml-3">{t("layout.logout")}</span>
                        )}
                      </div>
                    </button>
                  )}
                </div>
              </>
            )}
          </div>

          {/* Dimming overlay for mobile */}
          {isMobile && isSidebarOpen && (
            <div
              className="fixed inset-0 bg-black/50 z-30 transition-opacity duration-300"
              onClick={() => setIsSidebarOpen(false)}
            ></div>
          )}

          {/* Main Content Area - Modified for X-style shifting */}
          <div
            className={`flex justify-center h-full w-full transition-all duration-300 
              ${isMobile ? "transform" : "pl-16"} 
              ${isMobile && isSidebarOpen ? "translate-x-64" : "translate-x-0"}
              ${activePanel && !isMobile ? "lg:pr-[440px]" : ""}
              ${isIOS ? "pb-8" : "pb-3"}`}
          >
            <div className="flex-1 max-w-4xl flex flex-col overflow-hidden">
              {children}
            </div>
          </div>

          {/* Display active panel content for mobile - as a full-screen overlay */}
          {isMobile && activePanel && (
            <div className="fixed inset-0 z-40 bg-black/95 overflow-auto">
              <div className="p-4">
                <PanelSelector
                  activePanel={activePanel}
                  setActivePanel={setActivePanel}
                />
              </div>
            </div>
          )}

          {/* Panel Selector - Only on desktop */}
          {!isMobile && (
            <div
              className={`fixed right-0 top-16 bottom-0 z-30 transition-transform duration-300 ${
                activePanel ? "translate-x-0" : "translate-x-full"
              }`}
            >
              <PanelSelector
                activePanel={activePanel}
                setActivePanel={setActivePanel}
              />
            </div>
          )}
        </div>
      </div>
    </>
  );
}

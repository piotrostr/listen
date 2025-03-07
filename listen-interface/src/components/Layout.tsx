import { usePrivy } from "@privy-io/react-auth";
import { Link } from "@tanstack/react-router";
import { createContext, memo, useContext, useMemo, useState } from "react";
import { IoMenu } from "react-icons/io5";
import { UseBalanceReturnType } from "wagmi";
import ethereumIcon from "../assets/icons/ethereum.svg";
import { imageMap } from "../hooks/util";
import { Background } from "./Background";

import { useTranslation } from "react-i18next";
import { FaXTwitter } from "react-icons/fa6";
import { RxCross2 } from "react-icons/rx";
import { TbHistoryToggle } from "react-icons/tb";
import LanguageSwitcher from "./LanguageSwitcher";
import { PanelSelector } from "./PanelSelector";
import { RecentChats } from "./RecentChats";

function balanceToUI(balance: UseBalanceReturnType["data"]) {
  if (!balance?.value || !balance?.decimals) return 0;
  return Number(balance?.value) / 10 ** balance?.decimals;
}

// Memoize the NavLink component
const MemoizedNavLink = memo(function NavLink({
  to,
  icon: Icon,
  label,
  isSidebarOpen = true,
  isChat = false,
}: {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  isSidebarOpen?: boolean;
  isChat?: boolean;
}) {
  const setIsSidebarOpen = useContext(SidebarContext);

  return (
    <div className="relative">
      <Link
        to={to}
        className="flex items-center h-10 rounded-lg text-gray-300 hover:text-white hover:bg-purple-500/10 [&.active]:bg-purple-500/20 [&.active]:text-white transition-colors"
        onClick={() => {
          if (window.innerWidth < 1024) {
            setIsSidebarOpen(false);
          }
        }}
      >
        <div
          className={`flex items-center h-full ${
            isSidebarOpen ? "px-4 w-full" : "justify-center w-16"
          }`}
        >
          <Icon className="w-5 h-5 min-w-[20px]" />
          {isSidebarOpen && (
            <>
              <span className="ml-3 flex-1">{label}</span>
              {isChat && (
                <Link
                  to="/"
                  search={{ new: true }}
                  className="p-1 hover:bg-purple-500/20 rounded-full transition-colors"
                  title="New Chat"
                >
                  <svg
                    width="16"
                    height="16"
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
                </Link>
              )}
            </>
          )}
        </div>
      </Link>
      {isChat && isSidebarOpen && (
        <div className="mt-1">
          <RecentChats />
        </div>
      )}
    </div>
  );
});

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
      className="flex items-center h-10 rounded-lg text-gray-300 hover:text-white hover:bg-purple-500/10 transition-colors"
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

// Balance Display Component
export function BalanceDisplay({
  isSidebarOpen,
  solanaBalance,
  ethereumBalance,
}: {
  isSidebarOpen: boolean;
  solanaBalance?: number;
  ethereumBalance?: UseBalanceReturnType["data"];
}) {
  return (
    <div className="mt-8 space-y-1">
      <div
        className={`flex items-center h-10 ${
          isSidebarOpen ? "px-4" : "justify-center"
        }`}
      >
        <img src={imageMap.solana} alt="SOL" className="w-6 h-6 rounded-full" />
        {isSidebarOpen && (
          <span className="ml-3 text-sm text-gray-300">
            {solanaBalance?.toFixed(2) || "0.00"}
          </span>
        )}
      </div>
      <div
        className={`flex items-center h-10 ${
          isSidebarOpen ? "px-4" : "justify-center"
        }`}
      >
        <img src={ethereumIcon} alt="ETH" className="w-6 h-6 rounded-full" />
        {isSidebarOpen && (
          <span className="ml-3 text-sm text-gray-300">
            {balanceToUI(ethereumBalance)?.toFixed(4) || "0.0000"}
          </span>
        )}
      </div>
    </div>
  );
}

// Version Display Component
export function VersionAndLanguageDisplay() {
  return (
    <div className="flex justify-around items-center w-full">
      <span className="text-xs text-gray-400">version: 1.1.4</span>
      <LanguageSwitcher />
    </div>
  );
}

// Add this near the top of the file, after imports
const SidebarContext = createContext<(open: boolean) => void>(() => {});

// Move these outside the component as functions that take the translation function
function getNavItems(t: (key: string) => string) {
  return [
    {
      to: "/",
      icon: TbHistoryToggle,
      label: t("layout.chat_history"),
      isChat: true, // This will show the recent chats
    },
  ] as const;
}

function getBottomItems(t: (key: string) => string) {
  return [
    {
      href: "https://docs.listen-rs.com",
      icon: () => (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z" />
        </svg>
      ),
      label: t("layout.documentation"),
    },
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
  const { user, logout } = usePrivy();
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [activePanel, setActivePanel] = useState(null);

  const { t } = useTranslation();

  // Call the functions with the current translation function
  const NAV_ITEMS = getNavItems(t);
  const BOTTOM_ITEMS = getBottomItems(t);

  const handleSidebarToggle = (open: boolean) => {
    setIsSidebarOpen(open);
  };

  // Memoize the nav items - now depends on both isSidebarOpen and NAV_ITEMS (which depends on t)
  const memoizedNavItems = useMemo(
    () =>
      NAV_ITEMS.map((item) => (
        <MemoizedNavLink
          key={item.to}
          {...item}
          isSidebarOpen={isSidebarOpen}
        />
      )),
    [isSidebarOpen, NAV_ITEMS]
  );

  // Memoize the bottom items - now depends on both isSidebarOpen and BOTTOM_ITEMS (which depends on t)
  const memoizedBottomItems = useMemo(
    () =>
      BOTTOM_ITEMS.map((item) => (
        <MemoizedBottomLink
          key={item.href}
          {...item}
          isSidebarOpen={isSidebarOpen}
        />
      )),
    [isSidebarOpen, BOTTOM_ITEMS]
  );

  return (
    <SidebarContext.Provider value={handleSidebarToggle}>
      <div className="relative h-screen flex flex-col text-white overflow-hidden">
        <Background />

        {/* Mobile Header */}
        <div className="lg:hidden fixed top-0 left-0 right-0 h-16 z-50 bg-black/40 backdrop-blur-sm border-b border-purple-500/30 flex items-center justify-between px-4">
          <div className="flex items-center space-x-3">
            <img
              src="/listen-more.png"
              alt="Logo"
              className="w-8 h-8 rounded"
            />
            <span className="font-bold text-md lg:text-xl">listen-rs</span>
          </div>
          <button
            onClick={() => handleSidebarToggle(!isSidebarOpen)}
            className="p-2 rounded-lg hover:bg-purple-500/10 transition-colors"
          >
            {isSidebarOpen ? (
              <RxCross2 className="w-6 h-6" />
            ) : (
              <IoMenu className="w-6 h-6" />
            )}
          </button>
        </div>

        {/* Mobile Sidebar Overlay */}
        <div
          className={`lg:hidden fixed inset-0 z-40 bg-black/40 backdrop-blur-sm transition-opacity duration-300 ${
            isSidebarOpen ? "opacity-100" : "opacity-0 pointer-events-none"
          }`}
          onClick={() => handleSidebarToggle(false)}
        >
          <div
            className={`w-64 h-full bg-black/60 backdrop-blur-sm transition-transform duration-300 ${
              isSidebarOpen ? "translate-x-0" : "-translate-x-full"
            }`}
            onClick={(e) => e.stopPropagation()}
          >
            <div className="p-4 pt-20">
              <nav className="space-y-1">{memoizedNavItems}</nav>

              {/* Balance Display */}
              {/*isAuthenticated && (
                <BalanceDisplay
                  isSidebarOpen={true}
                  solanaBalance={solanaBalance}
                  ethereumBalance={ethereumBalance}
                />
              )*/}
            </div>

            {/* Bottom Items */}
            <div className="absolute bottom-0 left-0 right-0 mb-4 space-y-1">
              {isSidebarOpen && <VersionAndLanguageDisplay />}
              {memoizedBottomItems}
              {user && (
                <button
                  onClick={() => logout()}
                  className="flex items-center h-10 w-full rounded-lg text-gray-300 hover:text-white hover:bg-purple-500/10 transition-colors"
                >
                  <div className="flex items-center h-full px-4 w-full">
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
                    <span className="ml-3">{t("layout.logout")}</span>
                  </div>
                </button>
              )}
            </div>
          </div>
        </div>

        <div className="relative z-10 flex flex-1 h-full overflow-hidden">
          {/* Desktop Sidebar - Hidden on mobile */}
          <div className="hidden lg:flex w-64 border-r border-purple-500/30 bg-black/40 backdrop-blur-sm flex-col">
            {/* Logo section */}
            <div className="p-4">
              <div className="flex items-center">
                <div
                  className={`flex items-center ${isSidebarOpen ? "px-2" : "justify-center"}`}
                >
                  <img
                    src="/listen-more.png"
                    alt="Logo"
                    className="w-8 h-8 rounded"
                  />
                  {isSidebarOpen && (
                    <span className="ml-3 font-bold text-xl">listen-rs</span>
                  )}
                </div>
              </div>
            </div>

            {/* Navigation */}
            <div className="p-4">
              <nav className="space-y-1">{memoizedNavItems}</nav>

              {/* Balance Display */}
              {/*isAuthenticated && (
                <BalanceDisplay
                  isSidebarOpen={isSidebarOpen}
                  solanaBalance={solanaBalance}
                  ethereumBalance={ethereumBalance}
                />
              )*/}
            </div>

            {/* Bottom section */}
            <div className="mt-auto p-4 space-y-1">
              {isSidebarOpen && <VersionAndLanguageDisplay />}
              {memoizedBottomItems}
              {user && (
                <button
                  onClick={() => logout()}
                  className="flex items-center h-10 w-full rounded-lg text-gray-300 hover:text-white hover:bg-purple-500/10 transition-colors"
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
          </div>

          {/* Main Content */}
          <div className="flex-1 flex flex-col h-full overflow-hidden">
            {/* Add top padding only on mobile to account for the header */}
            <div className="flex-1 flex overflow-hidden pt-16 lg:pt-0">
              {/* Chat is always visible when no panel is active on mobile */}
              <div
                className={`flex-1 overflow-hidden ${activePanel && window.innerWidth < 1024 ? "hidden" : "block"}`}
              >
                {children}
              </div>

              {/* Right panel for toggleable components */}
              <PanelSelector
                activePanel={activePanel}
                setActivePanel={setActivePanel}
              />
            </div>
          </div>
        </div>
      </div>
      <head>
        <meta
          name="viewport"
          content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"
        />
      </head>
    </SidebarContext.Provider>
  );
}

import { usePrivy } from "@privy-io/react-auth";
import { Link } from "@tanstack/react-router";
import { createContext, useContext, useState } from "react";
import { IoChatboxOutline, IoWalletOutline } from "react-icons/io5";
import { UseBalanceReturnType } from "wagmi";
import ethereumIcon from "../assets/icons/ethereum.svg";
import { imageMap } from "../hooks/util";
import { Background } from "./Background";

import { BsLink } from "react-icons/bs";
import { FaXTwitter } from "react-icons/fa6";
import { IoChevronDown, IoMenu, IoSettingsOutline } from "react-icons/io5";
import { RxCross2, RxDashboard } from "react-icons/rx";
import { RecentChats } from "./RecentChats";

const NAV_ITEMS = [
  { to: "/screener", icon: RxDashboard, label: "Screener" },
  { to: "/portfolio", icon: IoWalletOutline, label: "Portfolio" },
  { to: "/pipelines", icon: BsLink, label: "Pipelines" },
  { to: "/settings", icon: IoSettingsOutline, label: "Settings" },
] as const;

const BOTTOM_ITEMS = [
  {
    href: "https://docs.listen-rs.com",
    icon: () => (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z" />
      </svg>
    ),
    label: "Documentation",
  },
  {
    href: "https://github.com/piotrostr/listen",
    icon: () => (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
      </svg>
    ),
    label: "GitHub",
  },
  {
    href: "https://x.com/listenonsol",
    icon: () => <FaXTwitter />,
    label: "Twitter",
  },
] as const;

function balanceToUI(balance: UseBalanceReturnType["data"]) {
  if (!balance?.value || !balance?.decimals) return 0;
  return Number(balance?.value) / 10 ** balance?.decimals;
}

// Navigation Link Component
function NavLink({
  to,
  icon: Icon,
  label,
  isSidebarOpen = true,
  isChat = false,
  isDrawerOpen = false,
  onToggleDrawer,
}: {
  to: string;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  isSidebarOpen?: boolean;
  isChat?: boolean;
  isDrawerOpen?: boolean;
  onToggleDrawer?: () => void;
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
                <div className="flex items-center space-x-2">
                  <Link
                    to="/chat"
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
                  <button
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      onToggleDrawer?.();
                    }}
                    className={`transition-transform ${isDrawerOpen ? "rotate-180" : ""}`}
                  >
                    <IoChevronDown className="w-4 h-4" />
                  </button>
                </div>
              )}
            </>
          )}
        </div>
      </Link>
      {isChat && isDrawerOpen && (
        <div className="mt-1">
          <RecentChats />
        </div>
      )}
    </div>
  );
}

// Bottom Link Component
function BottomLink({
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
}

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

// Add this near the top of the file, after imports
const SidebarContext = createContext<(open: boolean) => void>(() => {});

export function Layout({ children }: { children: React.ReactNode }) {
  const { user, logout } = usePrivy();
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isChatDrawerOpen, setIsChatDrawerOpen] = useState(false);

  // Add this handler function
  const handleSidebarToggle = (open: boolean) => {
    setIsSidebarOpen(open);
    if (!open) {
      setIsChatDrawerOpen(false);
    }
  };

  return (
    <SidebarContext.Provider value={handleSidebarToggle}>
      <div className="relative min-h-screen text-white">
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
              <nav className="space-y-1">
                {NAV_ITEMS.map((item) => (
                  <NavLink key={item.to} {...item} />
                ))}
                <NavLink
                  to="/chat"
                  icon={IoChatboxOutline}
                  label="Chat"
                  isSidebarOpen={true}
                  isChat={true}
                  isDrawerOpen={isChatDrawerOpen}
                  onToggleDrawer={() => setIsChatDrawerOpen(!isChatDrawerOpen)}
                />
              </nav>

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
              {BOTTOM_ITEMS.map((item) => (
                <BottomLink key={item.href} {...item} />
              ))}
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
                    <span className="ml-3">Logout</span>
                  </div>
                </button>
              )}
            </div>
          </div>
        </div>

        <div className="relative z-10 flex h-screen">
          {/* Desktop Sidebar - Hidden on mobile */}
          <div
            className={`hidden lg:flex ${
              isSidebarOpen ? "w-64" : "w-16"
            } border-r border-purple-500/30 bg-black/40 backdrop-blur-sm flex-col transition-all duration-300 group relative`}
          >
            {/* Desktop Toggle Button */}
            <div
              className={`absolute ${
                isSidebarOpen ? "right-0" : "left-0"
              } top-0 h-12 opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-center justify-center`}
            >
              <button
                onClick={() => handleSidebarToggle(!isSidebarOpen)}
                className="p-1.5 bg-black/40 backdrop-blur-sm hover:bg-purple-500/10 rounded-lg transition-colors"
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  style={{
                    transform: `rotate(${isSidebarOpen ? "180deg" : "0deg"})`,
                    transition: "transform 300ms ease-in-out",
                  }}
                >
                  <path d="M15 18l-6-6 6-6" />
                </svg>
              </button>
            </div>

            {/* Logo section */}
            <div className="p-4">
              <div className="flex items-center">
                <div
                  className={`flex items-center ${isSidebarOpen ? "px-2" : "justify-center"}`}
                >
                  <img
                    src="/listen-more.png"
                    alt="Logo"
                    className="w-8 h-8 rounded cursor-pointer"
                    onClick={() => handleSidebarToggle(!isSidebarOpen)}
                  />
                  {isSidebarOpen && (
                    <span className="ml-3 font-bold text-xl">listen-rs</span>
                  )}
                </div>
              </div>
            </div>

            {/* Navigation */}
            <div className="p-4">
              <nav className="space-y-1">
                {NAV_ITEMS.map((item) => (
                  <NavLink
                    key={item.to}
                    {...item}
                    isSidebarOpen={isSidebarOpen}
                  />
                ))}
                <NavLink
                  to="/chat"
                  icon={IoChatboxOutline}
                  label="Chat"
                  isSidebarOpen={isSidebarOpen}
                  isChat={true}
                  isDrawerOpen={isChatDrawerOpen}
                  onToggleDrawer={() => setIsChatDrawerOpen(!isChatDrawerOpen)}
                />
              </nav>

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
              {BOTTOM_ITEMS.map((item) => (
                <BottomLink
                  key={item.href}
                  {...item}
                  isSidebarOpen={isSidebarOpen}
                />
              ))}
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
                    {isSidebarOpen && <span className="ml-3">Logout</span>}
                  </div>
                </button>
              )}
            </div>
          </div>

          {/* Main Content */}
          <div className="flex-1 flex mt-16 lg:mt-0">
            <div className="flex-1 overflow-auto">{children}</div>
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

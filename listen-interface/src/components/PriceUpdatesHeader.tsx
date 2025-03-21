import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { BiSolidHide } from "react-icons/bi";
import { FaCircle, FaPause, FaRegStar } from "react-icons/fa";
import { GrClear } from "react-icons/gr";
import { IoRefreshOutline } from "react-icons/io5";
import { LuFilter, LuSearch } from "react-icons/lu";
import { useTokenStore } from "../store/tokenStore";

export function PriceUpdatesHeader() {
  const { t } = useTranslation();
  const [showFilterPopup, setShowFilterPopup] = useState(false);
  const [isSearchExpanded, setIsSearchExpanded] = useState(false);
  const searchRef = useRef<HTMLDivElement>(null);
  const {
    isListFrozen,
    showWatchlistOnly,
    showHiddenOnly,
    setShowWatchlistOnly,
    setShowHiddenOnly,
    marketCapFilter,
    setMarketCapFilter,
    volumeFilter,
    setVolumeFilter,
    setIsListFrozen,
    refreshTokenData,
    searchQuery,
    setSearchQuery,
  } = useTokenStore();

  const toggleFilterPopup = () => setShowFilterPopup(!showFilterPopup);

  const handleRefresh = () => {
    refreshTokenData();
  };

  // Handle click outside to collapse search
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        searchRef.current &&
        !searchRef.current.contains(event.target as Node) &&
        isSearchExpanded &&
        !searchQuery // Only collapse if search is empty
      ) {
        setIsSearchExpanded(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isSearchExpanded, searchQuery]);

  // Focus input when expanded
  useEffect(() => {
    if (isSearchExpanded) {
      const input = searchRef.current?.querySelector("input");
      if (input) {
        input.focus();
      }
    }
  }, [isSearchExpanded]);

  return (
    <div className="flex items-center justify-between gap-2 h-full">
      {isSearchExpanded ? (
        // When search is expanded, show only search input and frozen indicator
        <div className="flex items-center w-full gap-2">
          {isListFrozen && (
            <div
              className="flex-shrink-0 items-center gap-1 bg-black/60 border border-teal-400/30 rounded px-2 h-8 text-xs text-teal-300"
              onClick={() => setIsListFrozen(false)}
            >
              <FaPause className="text-teal-300 text-[10px]" />
            </div>
          )}

          <div ref={searchRef} className="h-8 w-full relative">
            <div className="absolute inset-y-0 left-0 flex items-center pl-2 pointer-events-none">
              <LuSearch className="w-4 h-4 text-gray-400" />
            </div>
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={t("price_updates.search") || "Search tokens..."}
              className="h-full rounded-lg pl-8 pr-8 py-1 text-sm bg-black/40 border border-[#2D2D2D] focus:outline-none w-full"
            />
            <button
              onClick={() => {
                setSearchQuery("");
                if (!searchQuery) {
                  setIsSearchExpanded(false);
                }
              }}
              className="absolute inset-y-0 right-0 flex items-center pr-2 text-gray-400 hover:text-white"
            >
              <GrClear className="w-3 h-3" />
            </button>
          </div>
        </div>
      ) : (
        // When search is not expanded, show all normal controls
        <>
          <div className="flex gap-2 h-full items-center">
            {isListFrozen && (
              <div
                className="flex items-center gap-1 bg-black/60 border border-teal-400/30 rounded px-2 h-8 text-xs text-teal-300"
                onClick={() => setIsListFrozen(false)}
              >
                <FaPause className="text-teal-300 text-[10px]" />
              </div>
            )}

            {/* Search button (collapsed) */}
            <div ref={searchRef} className="h-8">
              <button
                onClick={() => setIsSearchExpanded(true)}
                className="h-8 w-8 rounded-lg flex items-center justify-center text-gray-400 hover:text-white bg-black/40 border border-[#2D2D2D] hover:bg-[#2D2D2D] transition-all"
                title={t("price_updates.search") || "Search tokens"}
              >
                <LuSearch className="w-4 h-4" />
              </button>
            </div>

            {/* Filter button with popup */}
            <div className="relative">
              <button
                onClick={toggleFilterPopup}
                className={`h-8 rounded-lg px-3 text-sm flex items-center justify-center gap-2 border border-[#2D2D2D] ${
                  showFilterPopup ? "bg-[#2D2D2D]" : "bg-black/40"
                } hover:bg-[#2D2D2D] transition-all`}
                title="Filters"
              >
                <LuFilter size={14} />
                <span className="text-xs">{t("price_updates.filter")}</span>
              </button>

              {showFilterPopup && (
                <div className="absolute top-full left-0 mt-1 p-3 bg-black border border-[#2D2D2D] rounded-lg shadow-lg z-10 min-w-[200px]">
                  <div className="mb-3">
                    <div className="text-xs text-gray-400 mb-2">
                      {t("price_updates.volume")}
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() =>
                          setVolumeFilter(
                            volumeFilter === "bought" ? "all" : "bought"
                          )
                        }
                        className={`px-3 py-1 rounded text-sm flex items-center gap-2 ${
                          volumeFilter === "bought"
                            ? "bg-[#2D2D2D]"
                            : "bg-black/40"
                        } hover:bg-[#2D2D2D] transition-all`}
                      >
                        <FaCircle className="text-green-500 text-xs" />
                        <span>Buy</span>
                      </button>
                      <button
                        onClick={() =>
                          setVolumeFilter(
                            volumeFilter === "sold" ? "all" : "sold"
                          )
                        }
                        className={`px-3 py-1 rounded text-sm flex items-center gap-2 ${
                          volumeFilter === "sold"
                            ? "bg-[#2D2D2D]"
                            : "bg-black/40"
                        } hover:bg-[#2D2D2D] transition-all`}
                      >
                        <FaCircle className="text-red-500 text-xs" />
                        <span>{t("price_updates.sell")}</span>
                      </button>
                    </div>
                  </div>

                  <div>
                    <div className="text-xs text-gray-400 mb-2">
                      {t("price_updates.market_cap")}
                    </div>
                    <select
                      value={marketCapFilter}
                      onChange={(e) => setMarketCapFilter(e.target.value)}
                      className="bg-black/40 text-white rounded px-2 py-1 text-sm focus:outline-none w-full border border-[#2D2D2D]"
                    >
                      <option value="all">{t("price_updates.all")}</option>
                      <option value="under1m">&lt;$1M</option>
                      <option value="1mTo10m">$1M-$10M</option>
                      <option value="10mTo100m">$10M-$100M</option>
                      <option value="over100m">&gt;$100M</option>
                    </select>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Right side controls - only visible when search is not expanded */}
          <div className="flex gap-2 items-center">
            {/* Watchlist button */}
            <button
              onClick={() => setShowWatchlistOnly(!showWatchlistOnly)}
              className={`w-8 h-8 rounded-lg text-sm flex items-center justify-center ${
                showWatchlistOnly
                  ? "bg-yellow-500/20 text-yellow-300"
                  : "bg-black/40 text-gray-300"
              } hover:bg-[#2D2D2D] transition-all`}
              title={
                showWatchlistOnly ? "Show all tokens" : "Show watchlist only"
              }
            >
              <FaRegStar size={14} />
            </button>

            {/* Hidden button */}
            <button
              onClick={() => setShowHiddenOnly(!showHiddenOnly)}
              className={`w-8 h-8 rounded-lg text-sm flex items-center justify-center ${
                showHiddenOnly
                  ? "bg-red-500/20 text-red-300"
                  : "bg-black/40 text-gray-300"
              } hover:bg-[#2D2D2D] transition-all`}
              title={showHiddenOnly ? "Show all tokens" : "Show hidden tokens"}
            >
              <BiSolidHide size={14} />
            </button>

            {/* Refresh button */}
            <button
              onClick={handleRefresh}
              className="bg-black/40 text-white border border-[#2D2D2D] rounded-lg w-8 h-8 flex items-center justify-center hover:bg-white/10"
              title={t("portfolio.refresh")}
            >
              <IoRefreshOutline className="w-4 h-4" />
            </button>
          </div>
        </>
      )}
    </div>
  );
}

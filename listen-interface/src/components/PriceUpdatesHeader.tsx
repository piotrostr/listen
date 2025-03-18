import { useState } from "react";
import { useTranslation } from "react-i18next";
import { BiSolidHide } from "react-icons/bi";
import { FaCircle, FaPause, FaRegStar } from "react-icons/fa";
import { IoRefreshOutline } from "react-icons/io5";
import { LuFilter } from "react-icons/lu";
import { useTokenStore } from "../store/tokenStore";

export function PriceUpdatesHeader() {
  const { t } = useTranslation();
  const [showFilterPopup, setShowFilterPopup] = useState(false);
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
  } = useTokenStore();

  const toggleFilterPopup = () => setShowFilterPopup(!showFilterPopup);

  const handleRefresh = () => {
    refreshTokenData();
  };

  return (
    <div className="flex items-center justify-between gap-2 h-full">
      <div className="flex gap-2 h-full items-center">
        {isListFrozen && (
          <div
            className="flex items-center gap-1 bg-black/60 border border-teal-400/30 rounded px-2 h-8 text-xs text-teal-300"
            onClick={() => setIsListFrozen(false)}
          >
            <FaPause className="text-teal-300 text-[10px]" />
          </div>
        )}

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
                      volumeFilter === "bought" ? "bg-[#2D2D2D]" : "bg-black/40"
                    } hover:bg-[#2D2D2D] transition-all`}
                  >
                    <FaCircle className="text-green-500 text-xs" />
                    <span>Buy</span>
                  </button>
                  <button
                    onClick={() =>
                      setVolumeFilter(volumeFilter === "sold" ? "all" : "sold")
                    }
                    className={`px-3 py-1 rounded text-sm flex items-center gap-2 ${
                      volumeFilter === "sold" ? "bg-[#2D2D2D]" : "bg-black/40"
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

      {/* Controls on right side, matching Portfolio style */}
      <div className="flex gap-2 items-center">
        {/* Watchlist button */}
        <button
          onClick={() => setShowWatchlistOnly(!showWatchlistOnly)}
          className={`w-8 h-8 rounded-lg text-sm flex items-center justify-center ${
            showWatchlistOnly
              ? "bg-yellow-500/20 text-yellow-300"
              : "bg-black/40 text-gray-300"
          } hover:bg-[#2D2D2D] transition-all`}
          title={showWatchlistOnly ? "Show all tokens" : "Show watchlist only"}
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

        {/* Refresh button - now in the right group, matching Portfolio style */}
        <button
          onClick={handleRefresh}
          className="bg-black/40 text-white border border-[#2D2D2D] rounded-lg w-8 h-8 flex items-center justify-center hover:bg-white/10"
          title={t("portfolio.refresh")}
        >
          <IoRefreshOutline className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}

import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { BiSolidHide } from "react-icons/bi";
import { FaShoppingCart } from "react-icons/fa";
import { FaBoltLightning, FaRegStar, FaStar } from "react-icons/fa6";
import { HiOutlineSparkles } from "react-icons/hi2";
import { useChat } from "../contexts/ChatContext";
import { useModal } from "../contexts/ModalContext";
import { useListenMetadata } from "../hooks/useListenMetadata";
import { usePipelineExecution } from "../hooks/usePipelineExecution";
import i18n from "../i18n";
import { useSettingsStore } from "../store/settingsStore";
import { useTokenStore } from "../store/tokenStore";
import { TokenMarketData } from "../types/metadata";
import { Socials } from "./Socials";

interface TokenTileProps {
  token: TokenMarketData;
}

export function TokenTile({ token }: TokenTileProps) {
  const { openChart } = useModal();
  const { isLoading } = useChat();
  const { data: metadata } = useListenMetadata(token.pubkey);
  const { quickBuyAmount } = useSettingsStore();
  const [copied, setCopied] = useState(false);
  const { isExecuting, quickBuyToken } = usePipelineExecution();
  const [isHovered, setIsHovered] = useState(false);
  const [researchCooldown, setResearchCooldown] = useState(false);
  const { toggleWatchlist, toggleHidden, isWatchlisted } = useTokenStore();

  const { t } = useTranslation();

  useEffect(() => {
    if (copied) {
      setTimeout(() => setCopied(false), 1000);
    }
  }, [copied]);

  const handleBuy = async () => {
    await quickBuyToken(token.pubkey, quickBuyAmount);
  };

  const handleResearchClick = (e: React.MouseEvent) => {
    e.stopPropagation();

    if (researchCooldown) {
      e.preventDefault();
      return;
    }

    setResearchCooldown(true);

    setTimeout(() => {
      setResearchCooldown(false);
    }, 10000);
  };

  const handleHideToken = (e: React.MouseEvent) => {
    e.stopPropagation();
    toggleHidden(token.pubkey);
  };

  const isTokenWatchlisted = isWatchlisted(token.pubkey);

  const tokenSymbol = metadata?.mpl.symbol ?? token.name;
  const researchMessage =
    i18n.language === "en"
      ? `Listen, please research $${tokenSymbol} (${token.pubkey}). Provide it a score between 1 and 100 on how solid the narrative is.`
      : `听着，请研究 $${tokenSymbol} (${token.pubkey})。请给它的叙事可靠性评分，分数在1到100之间。`;

  return (
    <div
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <div className="p-3 sm:p-4 flex items-center justify-between hover:bg-black/50 transition-colors relative">
        <div className="flex items-center space-x-2 sm:space-x-4">
          <div className="flex items-center space-x-2 sm:space-x-3">
            {metadata?.mpl.ipfs_metadata?.image &&
              metadata.mpl.ipfs_metadata.image.startsWith("https://") && (
                <div className="w-8 h-8 sm:w-12 sm:h-12 relative rounded-full overflow-hidden">
                  <img
                    src={metadata.mpl.ipfs_metadata.image.replace(
                      "cf-ipfs.com",
                      "ipfs.io"
                    )}
                    alt={token.name}
                    className="w-full h-full object-cover"
                  />
                </div>
              )}
            <div>
              <div className="font-medium">
                <span className="inline-flex items-center text-sm sm:text-base">
                  <div
                    className="hover:text-blue-500 truncate max-w-[90px] sm:max-w-none cursor-pointer"
                    onClick={() => openChart(token.pubkey)}
                  >
                    {metadata?.mpl.symbol ?? token.name}
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      toggleWatchlist(token.pubkey);
                    }}
                    className={`ml-1 text-yellow-400 ${!isTokenWatchlisted && !isHovered ? "opacity-0" : "opacity-100"} transition-opacity`}
                    title={
                      isTokenWatchlisted
                        ? "Remove from watchlist"
                        : "Add to watchlist"
                    }
                  >
                    {isTokenWatchlisted ? (
                      <FaStar size={14} />
                    ) : (
                      <FaRegStar size={14} />
                    )}
                  </button>
                </span>
              </div>
              <Socials
                tokenMetadata={metadata ?? null}
                pubkey={token.pubkey}
                openChart={openChart}
              />
              <div className="text-xs sm:text-sm text-gray-500">
                {t("token_tile.market_cap")}: $
                {(token.marketCap / 1e6).toFixed(1)}M
              </div>
            </div>
            {researchCooldown ? (
              <div
                className={`p-2 ${isHovered ? "opacity-100" : "opacity-0"} opacity-50 bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 rounded-lg transition-all cursor-not-allowed`}
                title="Please wait before researching again"
              >
                <HiOutlineSparkles size={16} />
              </div>
            ) : (
              <Link
                to="/"
                search={{ new: true, message: researchMessage }}
                onClick={handleResearchClick}
                className={`p-2 ${isHovered ? "opacity-100" : "opacity-0"} hover:opacity-100 bg-indigo-500/20 hover:bg-indigo-500/40 text-indigo-300 border border-indigo-500/30 rounded-lg transition-all`}
                title="Research this token"
                disabled={isLoading}
              >
                <HiOutlineSparkles size={16} />
              </Link>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleHideToken}
            className={`${isHovered ? "opacity-100" : "opacity-0"} text-gray-500 transition-all hover:text-gray-400`}
            title="Hide this token"
          >
            <BiSolidHide size={16} />
          </button>
          <div className="text-right">
            <div className="flex flex-col">
              <span className="text-green-500 font-medium text-xs sm:text-base">
                +${parseFloat(token.buyVolume.toFixed(2)).toLocaleString()}
              </span>
              <span className="text-red-500 font-medium text-xs sm:text-base">
                -${parseFloat(token.sellVolume.toFixed(2)).toLocaleString()}
              </span>
            </div>
            <div className="flex justify-end items-center gap-2 mt-1">
              <div className="text-[10px] sm:text-xs text-gray-400 flex items-center gap-1">
                <FaBoltLightning />
                {token.uniqueAddresses.size}
              </div>
              <button
                onClick={handleBuy}
                disabled={isExecuting}
                className="px-2 py-1 bg-green-500/20 hover:bg-green-500/30 text-green-300 border border-green-500/30 rounded-lg text-xs transition-colors flex items-center gap-1"
              >
                {isExecuting ? (
                  <span className="animate-pulse">
                    {t("token_tile.executing")}
                  </span>
                ) : (
                  <>
                    <span>{quickBuyAmount}</span>
                    <FaShoppingCart size={12} />
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

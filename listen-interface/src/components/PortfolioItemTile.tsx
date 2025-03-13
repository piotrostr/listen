import { FaApplePay, FaShoppingCart } from "react-icons/fa";
import { IoArrowDown } from "react-icons/io5";
import { useModal } from "../contexts/ModalContext";
import { PortfolioItem } from "../hooks/types";

interface PortfolioItemTileProps {
  asset: PortfolioItem;
  onBuy: (asset: PortfolioItem) => void;
  onSell: (asset: PortfolioItem) => void;
  onTopup: () => void;
}

// Helper function to format amounts
const formatAmount = (amount: number): string => {
  if (amount >= 1_000_000_000) {
    return (amount / 1_000_000_000).toFixed(3) + "B";
  } else if (amount >= 1_000_000) {
    return (amount / 1_000_000).toFixed(3) + "M";
  } else if (amount >= 1000) {
    return amount.toFixed(3);
  } else {
    return amount.toFixed(6);
  }
};

// Helper function to format prices to 4 significant digits
const formatPrice = (price: number): string => {
  if (!price) return "0";

  // For numbers >= 0.001, use toFixed with appropriate decimal places
  if (price >= 0.001) {
    return price.toPrecision(4);
  }

  // For very small numbers, convert to string and find significant digits
  const priceStr = price.toString();
  const match = priceStr.match(/^0\.0*[1-9]/);
  if (match) {
    // Count leading zeros after decimal
    const leadingZeros = match[0].length - 3; // -3 for "0." and the first non-zero digit
    return price.toFixed(leadingZeros + 3); // Show 4 significant digits
  }

  return price.toPrecision(4);
};

export function PortfolioItemTile({
  asset,
  onBuy,
  onSell,
  onTopup,
}: PortfolioItemTileProps) {
  const { openChart } = useModal();
  return (
    <div className="p-3 sm:p-4 hover:bg-black/50 transition-colors">
      <div className="flex justify-between items-start mb-2">
        <div className="flex items-center gap-3">
          {asset.logoURI ? (
            <img
              src={asset.logoURI.replace("cf-ipfs.com", "ipfs.io")}
              alt={asset.symbol}
              className="w-12 h-12 rounded-full"
            />
          ) : (
            <div className="w-12 h-12 rounded-full bg-gray-200 flex items-center justify-center">
              <span className="text-gray-500 dark:text-gray-400">?</span>
            </div>
          )}
          <div>
            <h3 className="font-bold flex items-center gap-2">
              <div
                className="hover:text-blue-500 truncate max-w-[90px] sm:max-w-none cursor-pointer"
                onClick={() => openChart(asset.address)}
              >
                {asset.name}
              </div>
              <img
                src={
                  "https://dd.dexscreener.com/ds-data/chains/" +
                  asset.chain.toLowerCase() +
                  ".png"
                }
                className="w-4 h-4 hidden"
                alt={asset.chain}
              />
            </h3>
            <p className="text-sm text-gray-400">
              {formatAmount(asset.amount)} {asset.symbol}
            </p>
          </div>
        </div>
        <div className="text-right">
          <div className="flex items-center gap-2">
            {asset.chain === "solana" &&
              asset.address ===
                "So11111111111111111111111111111111111111112" && (
                <button
                  className="cursor-pointer border border-[#2D2D2D] rounded-full p-2 bg-transparent hover:bg-[#2D2D2D] transition-colors"
                  onClick={onTopup}
                >
                  <FaApplePay size={32} />
                </button>
              )}
            <div>
              <p className="font-bold">
                ${(asset.price * asset.amount).toFixed(2)}
              </p>
              <p className="text-sm text-gray-400">
                $
                {asset.address !==
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
                  ? formatPrice(asset.price)
                  : asset.price?.toFixed(2)}
              </p>
            </div>

            {/* Buy/Sell buttons - moved to right side */}
            {asset.chain === "solana" && (
              <div className="flex flex-col gap-2 ml-2">
                <button
                  onClick={() => onBuy(asset)}
                  className="px-2 py-1 bg-green-500/20 hover:bg-green-500/30 text-green-300 border border-green-500/30 rounded-lg text-xs transition-colors flex items-center justify-center"
                >
                  <FaShoppingCart size={12} />
                </button>
                <button
                  onClick={() => onSell(asset)}
                  className="px-2 py-1 bg-red-500/20 hover:bg-red-500/30 text-red-300 border border-red-500/30 rounded-lg text-xs transition-colors flex items-center justify-center"
                >
                  <IoArrowDown size={12} />
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

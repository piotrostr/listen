import { useModal } from "../contexts/ModalContext";
import { use24hOpenPrice } from "../hooks/use24hOpenPrice";
import { PortfolioItem } from "../lib/types";
import { ChainIcon } from "./ChainIcon";

interface PortfolioItemTileProps {
  asset: PortfolioItem;
  onBuy?: (asset: PortfolioItem) => void;
  onSell?: (asset: PortfolioItem) => void;
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

// Helper function to format percentage change
const formatPnL = (currentPrice: number, openPrice: number): string => {
  const pctChange = ((currentPrice - openPrice) / openPrice) * 100;
  const sign = pctChange >= 0 ? "+" : "";
  return `${sign}${pctChange.toFixed(2)}%`;
};

export function PortfolioItemTile({
  asset,
  onBuy,
  onSell,
}: PortfolioItemTileProps) {
  const { openChart } = useModal();
  const { data: openPriceData } = use24hOpenPrice(asset.address);

  const handleOpenChart = () => {
    openChart({
      mint: asset.address,
      chainId: asset.chain,
      onBuy: onBuy ? () => onBuy(asset) : undefined,
      onSell: onSell ? () => onSell(asset) : undefined,
      name: asset.name,
      symbol: asset.symbol,
      amount: asset.amount,
      logoURI: asset.logoURI,
      price: asset.price,
      decimals: asset.decimals,
    });
  };

  const pnlColor = openPriceData
    ? asset.price >= openPriceData.price
      ? "text-green-500"
      : "text-red-500"
    : "";

  return (
    <div
      className="p-3 sm:p-4 hover:bg-[#2d2d2d]/50 bg-[#2d2d2d]/20 transition-colors cursor-pointer rounded-2xl"
      onClick={handleOpenChart}
    >
      <div className="flex justify-between items-start">
        <div className="flex items-center gap-3">
          <div className="relative">
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
            {asset.chain !== "solana" && (
              <div className="absolute top-1 -left-1 z-10">
                <ChainIcon chainId={asset.chain} className="w-4 h-4" />
              </div>
            )}
          </div>
          <div>
            <h3 className="font-[400] flex items-center gap-2">
              <div className="truncate max-w-[100px] sm:max-w-none text-lg">
                {asset.name}
              </div>
            </h3>
            <p className="text-sm text-gray-400 font-dm-sans">
              {formatAmount(asset.amount)} {asset.symbol}
            </p>
          </div>
        </div>
        <div className="text-right">
          <div className="flex items-center gap-2">
            <div>
              <p className="font-bold font-dm-sans">
                ${(asset.price * asset.amount).toFixed(2)}
              </p>
              <p className={`text-sm font-dm-sans font-[500] ${pnlColor}`}>
                {openPriceData
                  ? formatPnL(asset.price, openPriceData.price)
                  : "$" + formatPrice(asset.price)}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

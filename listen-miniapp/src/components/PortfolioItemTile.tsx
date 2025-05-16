import React from "react";
import { useModal } from "../contexts/ModalContext";
import { PortfolioItem } from "../lib/types";
import { ChainIcon } from "./ChainIcon";

interface PortfolioItemTileProps {
  asset: PortfolioItem;
  onBuy?: (asset: PortfolioItem) => void;
  onSell?: (asset: PortfolioItem) => void;
}

const TokenLogo = ({ src, alt }: { src?: string; alt: string }) => {
  const [hasError, setHasError] = React.useState(false);
  const initials = alt.slice(0, 2).toUpperCase();

  if (!src || hasError) {
    return (
      <div className="w-12 h-12 rounded-full bg-[#1e1e21] flex items-center justify-center border-[1px] border-[#404040]">
        <span className="text-white">{initials}</span>
      </div>
    );
  }

  return (
    <img
      src={src.replace("cf-ipfs.com", "ipfs.io")}
      alt={alt}
      className="w-12 h-12 rounded-full"
      onError={() => setHasError(true)}
    />
  );
};

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
}: PortfolioItemTileProps) {
  const { openChart } = useModal();

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

  return (
    <div
      className="p-3 sm:p-4 hover:bg-[#2d2d2d]/50 bg-[#2d2d2d]/20 transition-colors cursor-pointer rounded-2xl"
      onClick={handleOpenChart}
    >
      <div className="flex justify-between items-start">
        <div className="flex items-center gap-3">
          <div className="relative">
            <TokenLogo src={asset.logoURI || undefined} alt={asset.symbol} />
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
              <p className="text-sm text-gray-400 font-dm-sans font-[500]">
                $
                {asset.address !==
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
                  ? formatPrice(asset.price)
                  : asset.price?.toFixed(2)}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

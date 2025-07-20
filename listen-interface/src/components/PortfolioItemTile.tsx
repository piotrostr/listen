import { useModal } from "../contexts/ModalContext";
import { PortfolioItem } from "../lib/types";
import { AggregatedPortfolioItem } from "../lib/portfolioHelpers";
import { formatAmountUI } from "../lib/util";
import { ChainIcon } from "./ChainIcon";

interface PortfolioItemTileProps {
  asset: PortfolioItem | AggregatedPortfolioItem;
  onBuy?: (asset: PortfolioItem | AggregatedPortfolioItem) => void;
  onSell?: (asset: PortfolioItem | AggregatedPortfolioItem) => void;
}

export function PortfolioItemTile({
  asset,
  onBuy,
  onSell,
}: PortfolioItemTileProps) {
  const { openChart } = useModal();

  const handleOpenChart = () => {
    // For aggregated items, use the first chain
    const chainId = "chains" in asset ? asset.chains[0] : asset.chain;

    openChart({
      mint: asset.address,
      chainId: chainId,
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

  const pnlColor =
    asset.priceChange24h >= 0 ? "text-[#8DFC63]" : "text-[#FF5C5C]";
  const pnlSign = asset.priceChange24h >= 0 ? "+" : "-";

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
          </div>
          <div>
            <h3 className="font-[400] flex items-center gap-2">
              <div className="truncate max-w-[100px] sm:max-w-none text-lg">
                {asset.name}
              </div>
              {/* Display chain icons next to the name for aggregated assets */}
              {"chains" in asset && asset.chains.length > 0 && (
                <div className="flex items-center gap-1">
                  {asset.chains.map((chain) => (
                    <ChainIcon
                      key={chain}
                      chainId={chain}
                      className="w-4 h-4"
                    />
                  ))}
                </div>
              )}
            </h3>
            <p className="text-sm text-gray-400 font-dm-sans">
              {formatAmountUI(asset.amount)} {asset.symbol}
            </p>
          </div>
        </div>
        <div className="text-right">
          <div className="flex items-center gap-2">
            <div>
              <p className="font-bold font-dm-sans">
                ${(asset.price * asset.amount).toFixed(2)}
              </p>
              {asset.priceChange24h != 0 && (
                <p className={`text-sm font-dm-sans font-[500] ${pnlColor}`}>
                  {pnlSign}
                  {Math.abs(asset.priceChange24h).toFixed(2)}%
                </p>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

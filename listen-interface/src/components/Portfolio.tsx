import { useState } from "react";
import { useEvmPortfolio } from "../hooks/useEvmPortfolioAlchemy";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { useSolanaPortfolio } from "../hooks/useSolanaPortfolio";
import { imageMap } from "../hooks/util";
import { CopyIcon } from "./CopyIcon";
import { PortfolioSkeleton } from "./PortfolioSkeleton";

export function Portfolio() {
  const { data: solanaAssets, isLoading: isLoadingSolana } =
    useSolanaPortfolio();
  const { data: evmAssets, isLoading: isLoadingEvm } = useEvmPortfolio();
  const isLoading = isLoadingSolana || isLoadingEvm;
  const { data: wallets } = usePrivyWallets();
  const [clickedSolana, setClickedSolana] = useState(false);
  const [clickedEvm, setClickedEvm] = useState(false);

  if (isLoading) {
    return <PortfolioSkeleton />;
  }

  const handleClickCopySolana = () => {
    if (!wallets) return;
    navigator.clipboard.writeText(wallets.solanaWallet.toString());
    setClickedSolana(true);
    setTimeout(() => setClickedSolana(false), 1000);
  };

  const handleClickCopyEvm = () => {
    if (!wallets) return;
    navigator.clipboard.writeText(wallets.evmWallet.toString());
    setClickedEvm(true);
    setTimeout(() => setClickedEvm(false), 1000);
  };

  const assets = [...(solanaAssets ?? []), ...(evmAssets ?? [])];

  return (
    <div className="h-full font-mono">
      <div className="flex lg:flex-row flex-col lg:justify-between lg:items-center p-4 lg:mt-3 lg:mb-3">
        <h2 className="text-xl font-bold lg:mb-0 mb-2">Portfolio</h2>
        <div className="flex lg:flex-row flex-col lg:items-center gap-2">
          <div className="flex items-center gap-2">
            <img
              src={imageMap["solana"]}
              alt="Solana"
              className="w-4 h-4 rounded-full"
            />
            {wallets?.solanaWallet?.toString().slice(0, 4)}...
            {wallets?.solanaWallet?.toString().slice(-5)}
            <div onClick={handleClickCopySolana} className="cursor-pointer">
              {clickedSolana ? <div> ✅</div> : <CopyIcon />}
            </div>
          </div>
          <div className="flex items-center gap-2">
            <img
              src={imageMap["eth"]}
              alt="Ethereum"
              className="w-4 h-4 rounded-full"
            />
            {wallets?.evmWallet?.toString().slice(0, 4)}...
            {wallets?.evmWallet?.toString().slice(-5)}
            <div onClick={handleClickCopyEvm} className="cursor-pointer">
              {clickedEvm ? <div> ✅</div> : <CopyIcon />}
            </div>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
        <div className="p-4 pt-0 space-y-4">
          {assets?.map((asset) => (
            <div
              key={asset.address}
              className="border border-purple-500/30 rounded-lg p-3 hover:bg-purple-900/20 transition-colors"
            >
              <div className="flex justify-between items-start mb-2">
                <div className="flex items-center gap-3">
                  <img
                    src={asset.logoURI}
                    alt={asset.symbol}
                    className="w-8 h-8 rounded-full"
                  />
                  <div>
                    <h3 className="font-bold flex items-center gap-2">
                      {asset.name}{" "}
                      <img
                        src={
                          "https://dd.dexscreener.com/ds-data/chains/" +
                          asset.chain.toLowerCase() +
                          ".png"
                        }
                        className="w-4 h-4"
                        alt={asset.chain}
                      />
                    </h3>
                    <p className="text-sm text-gray-400">${asset.symbol}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-bold">${asset.price?.toFixed(6)}</p>
                  <p className="text-sm text-gray-400">
                    ${(asset.price * asset.amount).toFixed(2)}
                  </p>
                </div>
              </div>
              <div className="text-sm text-gray-400">
                Holding: {asset.amount}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

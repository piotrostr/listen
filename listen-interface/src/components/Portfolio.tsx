import { useFundWallet } from "@privy-io/react-auth";
import { useSolanaFundingPlugin } from "@privy-io/react-auth/solana";
import { useState } from "react";
import { FaApplePay } from "react-icons/fa6";
import { useEvmPortfolio } from "../hooks/useEvmPortfolioAlchemy";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { useSolanaPortfolio } from "../hooks/useSolanaPortfolio";
import { imageMap } from "../hooks/util";
import { CopyIcon } from "./CopyIcon";
import { PortfolioSkeleton } from "./PortfolioSkeleton";

export function Portfolio() {
  const { fundWallet } = useFundWallet();
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

  useSolanaFundingPlugin();

  const handleTopup = async () => {
    if (!wallets) return;
    const wallet = wallets.solanaWallet.toString();
    console.log(wallet);
    await fundWallet(wallet, {
      defaultFundingMethod: "wallet",
      asset: "native-currency",
      config: {
        paymentMethod: "mobile_wallet",
      },
    });
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
          {assets
            ?.sort((a, b) => b.price * b.amount - a.price * a.amount)
            .map((asset) => (
              <div
                key={`${asset.address}-${asset.chain}`}
                className="border border-purple-500/30 rounded-lg p-3 hover:bg-purple-900/20 transition-colors"
              >
                <div className="flex justify-between items-start mb-2">
                  <div className="flex items-center gap-3">
                    {asset.logoURI ? (
                      <img
                        src={asset.logoURI}
                        alt={asset.symbol}
                        className="w-8 h-8 rounded-full"
                      />
                    ) : (
                      <div className="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center">
                        <span className="text-gray-500 dark:text-gray-400">
                          ?
                        </span>
                      </div>
                    )}
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
                    <div className="flex items-center gap-2">
                      {asset.address ===
                        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" && (
                        <button
                          className="cursor-pointer border border-purple-500/30 rounded-full p-2 bg-purple-500/10 hover:bg-purple-500/20 transition-colors"
                          onClick={handleTopup}
                          disabled={process.env.NODE_ENV === "production"}
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
                            ? asset.price?.toFixed(6)
                            : asset.price?.toFixed(2)}
                        </p>
                      </div>
                    </div>
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

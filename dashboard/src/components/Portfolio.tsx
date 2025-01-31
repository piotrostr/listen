import { useState } from "react";
import { usePortfolio } from "../hooks/usePortfolio";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { ChatSelector } from "./ChatSelector";
import { useChatType } from "../hooks/useChatType";

export function PortfolioSkeleton() {
  return (
    <div className="h-[70vh] font-mono">
      <div className="h-full border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm overflow-hidden flex flex-col">
        <div className="flex flex-row justify-between items-center p-4">
          <div className="h-7 w-28 bg-purple-500/20 rounded animate-pulse" />
          <div className="flex items-center gap-2">
            <div className="h-6 w-32 bg-purple-500/20 rounded animate-pulse" />
            <div className="h-4 w-4 bg-purple-500/20 rounded animate-pulse" />
          </div>
        </div>

        <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
          <div className="p-4 pt-0 space-y-4">
            {[1, 2, 3, 4].map((index) => (
              <div
                key={index}
                className="border border-purple-500/30 rounded-lg p-3"
              >
                <div className="flex justify-between items-start mb-2">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-full bg-purple-500/20 animate-pulse" />
                    <div>
                      <div className="h-5 w-24 bg-purple-500/20 rounded animate-pulse mb-1" />
                      <div className="h-4 w-16 bg-purple-500/20 rounded animate-pulse" />
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="h-5 w-20 bg-purple-500/20 rounded animate-pulse mb-1" />
                    <div className="h-4 w-24 bg-purple-500/20 rounded animate-pulse" />
                  </div>
                </div>
                <div className="h-4 w-32 bg-purple-500/20 rounded animate-pulse mt-2" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

const CopyIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    strokeWidth={1.5}
    stroke="currentColor"
    className="w-4 h-4 cursor-pointer"
  >
    <path d="M15.75 17.25v3.375c0 .621-.504 1.125-1.125 1.125h-9.75a1.125 1.125 0 01-1.125-1.125V7.875c0-.621.504-1.125 1.125-1.125H6.75a9.06 9.06 0 011.5.124m7.5 10.376h3.375c.621 0 1.125-.504 1.125-1.125V11.25c0-4.46-3.243-8.161-7.5-8.876a9.06 9.06 0 00-1.5-.124H9.375c-.621 0-1.125.504-1.125 1.125v3.5m7.5 10.375H9.375a1.125 1.125 0 01-1.125-1.125v-9.25m12 6.625v-1.875a3.375 3.375 0 00-3.375-3.375h-1.5a1.125 1.125 0 01-1.125-1.125v-1.5a3.375 3.375 0 00-3.375-3.375H9.75" />
  </svg>
);

export function Portfolio() {
  const { data: assets, isLoading } = usePortfolio();
  const { data: wallets } = usePrivyWallets();
  const [clicked, setClicked] = useState(false);
  const { chatType, setChatType } = useChatType();

  if (isLoading) {
    return <PortfolioSkeleton />;
  }

  const handleClickCopy = () => {
    if (!wallets) return;
    navigator.clipboard.writeText(
      chatType === "solana" || chatType === "pump"
        ? wallets.solanaWallet
        : wallets.evmWallet,
    );
    setClicked(true);
    setTimeout(() => setClicked(false), 1000);
  };

  return (
    <div className="h-[70vh] font-mono">
      <div className="h-full border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm overflow-hidden flex flex-col">
        <div className="flex flex-row justify-between items-center p-4">
          <h2 className="text-xl font-bold">Portfolio</h2>
          <div className="flex items-center gap-2">
            {(chatType === "solana" || chatType === "pump") && (
              <>
                <div>
                  {wallets?.solanaWallet?.toString().slice(0, 4)}...
                  {wallets?.solanaWallet?.toString().slice(-5)}
                </div>
                <div onClick={handleClickCopy} className="cursor-pointer">
                  {clicked ? <div> ✅</div> : <CopyIcon />}
                </div>
              </>
            )}
            {chatType === "evm" && (
              <>
                <div>
                  {wallets?.evmWallet?.toString().slice(0, 4)}...
                  {wallets?.evmWallet?.toString().slice(-5)}
                </div>
                <div onClick={handleClickCopy} className="cursor-pointer">
                  {clicked ? <div> ✅</div> : <CopyIcon />}
                </div>
              </>
            )}
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
                      <h3 className="font-bold">{asset.name}</h3>
                      <p className="text-sm text-gray-400">${asset.symbol}</p>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="font-bold">
                      ${asset.price?.toLocaleString()}
                    </p>
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
        <ChatSelector selectedChat={chatType} onSelectChat={setChatType} />
      </div>
    </div>
  );
}

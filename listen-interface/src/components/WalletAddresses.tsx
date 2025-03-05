import {
  useFundWallet,
  usePrivy,
  useSolanaWallets,
} from "@privy-io/react-auth";
import { useFundWallet as useFundSolanaWallet } from "@privy-io/react-auth/solana";
import { useState } from "react";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { imageMap } from "../hooks/util";
import { CopyIcon } from "./CopyIcon";

export function WalletAddresses() {
  const { data: wallets } = usePrivyWallets();
  const { exportWallet: exportEvmWallet } = usePrivy();
  const { exportWallet: exportSolanaWallet } = useSolanaWallets();
  const { fundWallet: fundEvmWallet } = useFundWallet();
  const { fundWallet: fundSolanaWallet } = useFundSolanaWallet();
  const [clickedSolana, setClickedSolana] = useState(false);
  const [clickedEvm, setClickedEvm] = useState(false);

  const handleClickCopySolana = () => {
    if (!wallets?.solanaWallet) return;
    navigator.clipboard.writeText(wallets.solanaWallet.toString());
    setClickedSolana(true);
    setTimeout(() => setClickedSolana(false), 1000);
  };

  const handleClickCopyEvm = () => {
    if (!wallets?.evmWallet) return;
    navigator.clipboard.writeText(wallets.evmWallet.toString());
    setClickedEvm(true);
    setTimeout(() => setClickedEvm(false), 1000);
  };

  const formatAddress = (address: string) => {
    return `${address.slice(0, 4)}...${address.slice(-4)}`;
  };

  return (
    <div className="space-y-2">
      {wallets?.solanaWallet && (
        <div className="border border-purple-500/30 rounded-lg p-3 hover:bg-purple-900/20 transition-colors backdrop-blur-sm">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <img
                src={imageMap["solana"]}
                alt="Solana"
                className="w-4 h-4 sm:w-6 sm:h-6 rounded-full"
              />
              <span className="font-bold text-sm sm:text-base">
                Solana Wallet
              </span>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => exportSolanaWallet()}
                className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-purple-500/10"
              >
                Export
              </button>
              <button
                onClick={() => fundSolanaWallet(wallets.solanaWallet!)}
                disabled={!wallets.solanaWallet}
                className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-purple-500/10"
              >
                Fund
              </button>
            </div>
          </div>
          <div className="flex items-center justify-between gap-2 bg-black/40 p-2 rounded font-mono text-xs sm:text-sm">
            <div className="truncate">
              <span className="hidden sm:inline">
                {wallets.solanaWallet.toString()}
              </span>
              <span className="sm:hidden">
                {formatAddress(wallets.solanaWallet.toString())}
              </span>
            </div>
            <div
              onClick={handleClickCopySolana}
              className="cursor-pointer flex-shrink-0 hover:text-blue-400"
            >
              {clickedSolana ? <div>✅</div> : <CopyIcon />}
            </div>
          </div>
        </div>
      )}

      {wallets?.evmWallet && (
        <div className="border border-purple-500/30 rounded-lg p-3 hover:bg-purple-900/20 transition-colors backdrop-blur-sm">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <img
                src={imageMap["eth"]}
                alt="Ethereum"
                className="w-4 h-4 sm:w-6 sm:h-6 rounded-full"
              />
              <span className="font-bold text-sm sm:text-base">EVM Wallet</span>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={exportEvmWallet}
                className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-purple-500/10"
              >
                Export
              </button>
              <button
                onClick={() => fundEvmWallet(wallets.evmWallet!)}
                disabled={true}
                className="p-2 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-purple-500/10 disabled:opacity-50"
              >
                Fund
              </button>
            </div>
          </div>
          <div className="flex items-center justify-between gap-2 bg-black/40 p-2 rounded font-mono text-xs sm:text-sm">
            <div className="truncate">
              <span className="hidden sm:inline">
                {wallets.evmWallet.toString()}
              </span>
              <span className="sm:hidden">
                {formatAddress(wallets.evmWallet.toString())}
              </span>
            </div>
            <div
              onClick={handleClickCopyEvm}
              className="cursor-pointer flex-shrink-0 hover:text-blue-400"
            >
              {clickedEvm ? <div>✅</div> : <CopyIcon />}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

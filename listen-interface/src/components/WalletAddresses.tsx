import {
  useFundWallet,
  usePrivy,
  useSolanaWallets,
} from "@privy-io/react-auth";
import { useFundWallet as useFundSolanaWallet } from "@privy-io/react-auth/solana";
import { useState } from "react";
import { useTranslation } from "react-i18next";
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

  const { t } = useTranslation();

  return (
    <div className="space-y-2">
      {wallets?.solanaWallet && (
        <div className="rounded-lg p-3 transition-colors backdrop-blur-sm">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <img
                src={imageMap["solana"]}
                alt="Solana"
                className="w-4 h-4 sm:w-6 sm:h-6 rounded-full"
              />
              <span className="font-bold text-sm sm:text-base">
                {t("wallet_addresses.solana_wallet")}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => exportSolanaWallet()}
                className="p-2 border-2 border-[#2d2d2d] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-[#2D2D2D]"
              >
                {t("wallet_addresses.export")}
              </button>
              <button
                onClick={() => fundSolanaWallet(wallets.solanaWallet!)}
                disabled={!wallets.solanaWallet}
                className="p-2 border-2 border-[#2d2d2d] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-[#2D2D2D]"
              >
                {t("wallet_addresses.fund")}
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
              className="cursor-pointer flex-shrink-0 hover:text-white"
            >
              {clickedSolana ? <div>✅</div> : <CopyIcon />}
            </div>
          </div>
        </div>
      )}

      {wallets?.evmWallet && (
        <div className="rounded-lg p-3 transition-colors backdrop-blur-sm">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <img
                src={imageMap["eth"]}
                alt="Ethereum"
                className="w-4 h-4 sm:w-6 sm:h-6 rounded-full"
              />
              <span className="font-bold text-sm sm:text-base">
                {t("wallet_addresses.evm_wallet")}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={exportEvmWallet}
                className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-[#2D2D2D]"
              >
                {t("wallet_addresses.export")}
              </button>
              <button
                onClick={() => fundEvmWallet(wallets.evmWallet!)}
                disabled={true}
                className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-xs sm:text-sm hover:bg-[#2D2D2D] disabled:opacity-50"
              >
                {t("wallet_addresses.fund")}
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

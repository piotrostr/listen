import { UseBalanceReturnType } from "wagmi";
import bscIcon from "../assets/icons/bsc.svg";
import ethereumIcon from "../assets/icons/ethereum.png";
import { useToken } from "../hooks/useToken";
import { imageMap } from "../lib/util";

export const Balance = ({
  solanaBalance,
  ethereumBalance,
}: {
  solanaBalance: number | undefined;
  ethereumBalance: number | undefined;
}) => {
  return (
    <div className="flex flex-row gap-1">
      <SolanaBalance solanaBalance={solanaBalance} />
      <EthereumBalance ethereumBalance={ethereumBalance?.toString() || "0"} />
    </div>
  );
};

export const SolanaBalance = ({
  solanaBalance,
}: {
  solanaBalance: number | undefined;
}) => {
  const isNaN = Number.isNaN(solanaBalance);
  return (
    <div className="flex items-center gap-2 mr-4">
      <img src={imageMap.solana} alt="SOL" className="w-6 h-6 rounded-full" />
      <span className="text-sm text-gray-300">
        {isNaN ? "0.00" : solanaBalance?.toFixed(2) || "0.00"}
      </span>
    </div>
  );
};

export const SplTokenBalance = ({
  amount,
  decimals,
  mint,
}: {
  amount: string | undefined;
  decimals: number | undefined;
  mint: string;
}) => {
  const { data: token } = useToken(mint);
  let numAmount = Number(amount);
  let numDecimals = Number(decimals);
  if (isNaN(numAmount)) {
    numAmount = 0;
  }
  if (isNaN(numDecimals)) {
    numDecimals = 0;
  }
  let uiAmount = numAmount / 10 ** numDecimals;
  if (isNaN(uiAmount)) {
    uiAmount = 0;
  }
  return (
    <div className="flex items-center gap-2 mr-4">
      {token?.logoURI ? (
        <img
          src={token?.logoURI}
          alt={token?.name ?? ""}
          className="w-6 h-6 rounded-full"
        />
      ) : (
        <div className="w-6 h-6 rounded-full bg-gray-200 flex items-center justify-center">
          <span className="text-gray-500 dark:text-gray-400">?</span>
        </div>
      )}
      <span className="text-sm text-gray-300">
        {uiAmount.toFixed(2) || "0.00"}
      </span>
    </div>
  );
};

export const EthereumBalance = ({
  ethereumBalance,
  chainId,
}: {
  ethereumBalance: string;
  chainId?: number;
}) => {
  return (
    <div className="flex items-center gap-2 mr-4">
      {chainId === 56 ? (
        <img src={bscIcon} alt="BSC" className="w-6 h-6 rounded-full" />
      ) : (
        <img src={ethereumIcon} alt="ETH" className="w-6 h-6 rounded-full" />
      )}
      <span className="text-sm text-gray-300">
        {(parseInt(ethereumBalance) / 10 ** 18).toFixed(5) || "0.00000"}
      </span>
      {chainId && (
        <span className="text-sm text-gray-300">
          ({chainIdToName(chainId)})
        </span>
      )}
    </div>
  );
};

const chainIdToName = (chainId: number) => {
  switch (chainId) {
    case 1:
      return "Mainnet";
    case 8453:
      return "Base";
    case 42161:
      return "Arbitrum";
    case 56:
      return "BSC";
    default:
      return "Unknown";
  }
};

export function balanceToUI(balance: UseBalanceReturnType["data"]) {
  if (!balance?.value || !balance?.decimals) return 0;
  return Number(balance?.value) / 10 ** balance?.decimals;
}

import { UseBalanceReturnType } from "wagmi";
import ethereumIcon from "../assets/icons/ethereum.svg";
import { imageMap } from "../hooks/util";

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
      <EthereumBalance ethereumBalance={ethereumBalance} />
    </div>
  );
};

export const SolanaBalance = ({
  solanaBalance,
}: {
  solanaBalance: number | undefined;
}) => {
  return (
    <div className="flex items-center gap-2 mr-4">
      <img src={imageMap.solana} alt="SOL" className="w-6 h-6 rounded-full" />
      <span className="text-sm text-gray-300">
        {solanaBalance?.toFixed(2) || "0.00"}
      </span>
    </div>
  );
};

export const EthereumBalance = ({
  ethereumBalance,
}: {
  ethereumBalance: number | undefined;
}) => {
  return (
    <div className="flex items-center gap-2 mr-4">
      <img src={ethereumIcon} alt="ETH" className="w-6 h-6 rounded-full" />
      <span className="text-sm text-gray-300">
        {ethereumBalance?.toFixed(4) || "0.0000"}
      </span>
    </div>
  );
};

export function balanceToUI(balance: UseBalanceReturnType["data"]) {
  if (!balance?.value || !balance?.decimals) return 0;
  return Number(balance?.value) / 10 ** balance?.decimals;
}

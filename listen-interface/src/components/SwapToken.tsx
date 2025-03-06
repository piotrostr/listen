import { useTranslation } from "react-i18next";

interface SwapTokenProps {
  image?: string | null;
  name?: string;
  amount?: string;
  chainId?: string | null;
  address?: string;
  showAmount?: boolean;
}

export const SwapToken = ({
  image,
  name,
  amount,
  chainId,
  address,
  showAmount = false,
}: SwapTokenProps) => {
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-3">
      <div className="flex flex-col">
        {image && (
          <img src={image} alt={name} className="w-8 h-8 rounded-full" />
        )}
      </div>
      <div>
        <div className="flex items-center gap-2">
          <div className="font-bold text-purple-100 text-base sm:text-lg">
            {name}
          </div>
          {chainId && (
            <img
              src={`https://dd.dexscreener.com/ds-data/chains/${chainId.toLowerCase()}.png`}
              alt={chainId}
              className="w-3 h-3 rounded-full"
            />
          )}
        </div>
        {showAmount && amount && (
          <div className="text-xs sm:text-sm text-purple-300">
            {t("pipelines.amount")}: {amount}
          </div>
        )}
        {address && (
          <div className="text-xs sm:text-sm text-gray-400 flex items-center gap-1">
            {address.slice(0, 4)}...{address.slice(-4)}
          </div>
        )}
      </div>
    </div>
  );
};

interface SwapTokenProps {
  image?: string | null;
  name?: string;
  amount?: string;
  chainId?: string | null;
  address?: string;
  showAmount?: boolean;
  compact: boolean;
}

export const SwapToken = ({
  image,
  name,
  amount,
  chainId,
  address,
  compact,
  showAmount = false,
}: SwapTokenProps) => {
  if (compact) {
    return <SwapTokenCompact image={image} name={name} amount={amount} />;
  }
  return (
    <div className="flex items-center gap-3">
      <div className="flex flex-col">
        {image && (
          <img
            src={image.replace("cf-ipfs.com", "ipfs.io")}
            alt={name}
            className="w-8 h-8 rounded-full"
          />
        )}
      </div>
      <div>
        <div className="flex items-center gap-2">
          <div className="font-bold text-base sm:text-lg">{name}</div>
          {chainId && (
            <img
              src={`https://dd.dexscreener.com/ds-data/chains/${chainId.toLowerCase()}.png`}
              alt={chainId}
              className="w-3 h-3 rounded-full"
            />
          )}
        </div>
        {showAmount && amount && (
          <div className="text-xs sm:text-sm">{amount}</div>
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

export const SwapTokenCompact = ({
  image,
  name,
  amount,
}: {
  image: string | null | undefined;
  name: string | null | undefined;
  amount: string | null | undefined;
}) => {
  return (
    <div className="flex flex-col items-center gap-1">
      <div className="text-sm font-medium">{name ?? ""}</div>
      {image && (
        <img
          src={image.replace("cf-ipfs.com", "ipfs.io")}
          alt={name ?? ""}
          className="w-10 h-10 rounded-full"
        />
      )}
      <span className="text-sm font-medium">{amount ?? ""}</span>
    </div>
  );
};

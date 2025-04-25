import ethIcon from "../assets/icons/ethereum.png";
import { chainIdNumericToChainId } from "../hooks/util";

interface ChainIconProps {
  chainId: string | number;
  className?: string;
}

export const ChainIcon = ({
  chainId,
  className = "w-4 h-4",
}: ChainIconProps) => {
  const chainIdentifier =
    typeof chainId === "string" ? chainId : chainIdNumericToChainId(chainId);

  if (chainIdentifier === "base") {
    return (
      <div className={`rounded-full bg-white ${className}`}>
        <svg
          viewBox="0 0 256 256"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            fillRule="evenodd"
            clipRule="evenodd"
            d="M256 128C256 198.692 198.592 256 127.777 256C60.5909 256 5.47394 204.417 0 138.759H169.482V117.24H0C5.47394 51.583 60.5909 0 127.777 0C198.592 0 256 57.3074 256 128Z"
            fill="#0052FF"
          />
        </svg>
      </div>
    );
  }
  if (chainIdentifier === "ethereum") {
    return (
      <img
        src={ethIcon}
        alt={chainIdentifier}
        className={`rounded-full ${className}`}
      />
    );
  }
  return (
    <img
      src={`https://dd.dexscreener.com/ds-data/chains/${chainIdentifier}.png`}
      alt={chainIdentifier}
      className={`rounded-full ${className}`}
    />
  );
};

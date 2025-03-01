import { useListenMetadata } from "../hooks/useListenMetadata";
import { formatAmount, imageMap } from "../hooks/util";
import { JupiterQuoteResponse } from "../types/quote";

interface JupiterQuoteDisplayProps {
  quote: JupiterQuoteResponse;
}

export const JupiterQuoteDisplay = ({ quote }: JupiterQuoteDisplayProps) => {
  // Fetch token metadata for images
  const inputTokenMetadata = useListenMetadata(quote.inputMint);
  const outputTokenMetadata = useListenMetadata(quote.outputMint);

  const inputSymbol = inputTokenMetadata.data?.mpl?.symbol || "Unknown";
  const outputSymbol = outputTokenMetadata.data?.mpl?.symbol || "Unknown";

  const inputImage =
    quote.inputMint in imageMap
      ? imageMap[quote.inputMint as keyof typeof imageMap]
      : inputTokenMetadata.data?.mpl?.ipfs_metadata?.image;

  const outputImage =
    quote.outputMint in imageMap
      ? imageMap[quote.outputMint as keyof typeof imageMap]
      : outputTokenMetadata.data?.mpl?.ipfs_metadata?.image;

  // Calculate formatted amounts based on token decimals
  const inputDecimals = inputTokenMetadata.data?.spl?.decimals || 9;
  const outputDecimals = outputTokenMetadata.data?.spl?.decimals || 9;

  // Format price impact as percentage
  const priceImpactPercentage = (
    parseFloat(quote.priceImpactPct) * 100
  ).toFixed(2);

  return (
    <div className="border border-purple-500/30 rounded-lg p-4 bg-black/40 backdrop-blur-sm">
      <div className="flex items-center gap-4">
        {/* Input Token */}
        <div className="flex-1">
          <div className="flex items-center gap-3">
            {inputImage ? (
              <img
                src={inputImage}
                alt={inputSymbol}
                className="w-10 h-10 rounded-full"
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = "none";
                }}
              />
            ) : (
              <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center text-purple-300">
                {inputSymbol.slice(0, 2)}
              </div>
            )}
            <div>
              <div className="font-bold text-purple-100 flex items-center gap-2">
                {inputSymbol}
              </div>
              <div className="text-sm text-purple-300">
                {formatAmount(quote.inAmount, inputDecimals)}
              </div>
              <div className="text-xs text-purple-300/70">
                {quote.inputMint.slice(0, 6)}...{quote.inputMint.slice(-4)}
              </div>
            </div>
          </div>
        </div>

        {/* Arrow */}
        <div className="text-purple-500">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth={1.5}
            stroke="currentColor"
            className="w-6 h-6"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"
            />
          </svg>
        </div>

        {/* Output Token */}
        <div className="flex-1">
          <div className="flex items-center gap-3">
            {outputImage ? (
              <img
                src={outputImage}
                alt={outputSymbol}
                className="w-10 h-10 rounded-full"
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = "none";
                }}
              />
            ) : (
              <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center text-purple-300">
                {outputSymbol.slice(0, 2)}
              </div>
            )}
            <div>
              <div className="font-bold text-purple-100 flex items-center gap-2">
                {outputSymbol}
              </div>
              <div className="text-sm text-purple-300">
                {formatAmount(quote.outAmount, outputDecimals)}
              </div>
              <div className="text-xs text-purple-300/70">
                {quote.outputMint.slice(0, 6)}...{quote.outputMint.slice(-4)}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Transaction Details */}
      <div className="mt-3 pt-3 border-t border-purple-500/30">
        <div className="flex justify-between items-start">
          <div>
            <div className="text-sm text-purple-300">
              Route Type: {quote.routePlan.length > 1 ? "Multi-hop" : "Direct"}
            </div>
            <div className="text-sm text-purple-300">
              Price Impact: {priceImpactPercentage}%
            </div>
          </div>
          <div className="text-right">
            <div className="text-sm text-purple-300">
              Slippage: {(quote.slippageBps / 100).toFixed(2)}%
            </div>
            <div className="text-sm text-purple-300">
              Min received:{" "}
              {formatAmount(quote.otherAmountThreshold, outputDecimals)}{" "}
              {outputSymbol}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

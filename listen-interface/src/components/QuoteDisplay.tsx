import { useToken } from "../hooks/useToken";
import { chainIdNumericToChainId, formatAmount, imageMap } from "../hooks/util";
import { QuoteResponse } from "../types/quote";

interface QuoteDisplayProps {
  quote: QuoteResponse;
}

const getTokenDecimals = (token: string, quote: QuoteResponse): number => {
  // Check if token matches from/to tokens
  if (token === quote.from.token) {
    return quote.from.decimals;
  }
  if (token === quote.to.token) {
    return quote.to.decimals;
  }

  // Common tokens
  switch (token) {
    case "USDC":
      return 6;
    case "SOL":
      return 9;
    default:
      return 18; // default for ETH/native tokens
  }
};

export const QuoteDisplay = ({ quote }: QuoteDisplayProps) => {
  const inputAmount = quote.from.amount;
  const outputAmount = quote.to.amount;
  const inputTokenDecimals = quote.from.decimals;
  const outputTokenDecimals = quote.to.decimals;

  // Get chain IDs
  const fromChain = chainIdNumericToChainId(quote.from.chain_id);
  const toChain = chainIdNumericToChainId(quote.to.chain_id);

  // Fetch token metadata for images
  const inputToken = useToken(
    quote.from.address,
    quote.from.chain_id.toString()
  );
  const outputToken = useToken(quote.to.address, quote.to.chain_id.toString());

  const inputImage = inputToken.data?.logoURI;
  const outputImage =
    outputToken.data?.logoURI ||
    imageMap[quote.to.token as keyof typeof imageMap];

  return (
    <div className="border border-[#2D2D2D] rounded-lg p-4 bg-black/40 backdrop-blur-sm">
      <div className="flex items-center gap-4">
        {/* Input Token */}
        <div className="flex-1">
          <div className="flex items-center gap-3">
            {inputImage ? (
              <img
                src={inputImage}
                alt={quote.from.token}
                className="w-10 h-10 rounded-full"
              />
            ) : (
              <div className="w-10 h-10 rounded-full flex items-center justify-center">
                {quote.from.token.slice(0, 2)}
              </div>
            )}
            <div>
              <div className="font-bold flex items-center gap-2">
                {quote.from.token}
                <img
                  src={`https://dd.dexscreener.com/ds-data/chains/${fromChain}.png`}
                  alt={fromChain}
                  className="w-4 h-4 rounded-full"
                />
              </div>
              <div className="text-sm">
                {formatAmount(inputAmount, inputTokenDecimals)}
              </div>
              <div className="text-xs">
                {quote.from.address.slice(0, 6)}...
                {quote.from.address.slice(-4)}
              </div>
            </div>
          </div>
        </div>

        {/* Arrow */}
        <div className="text-white">
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
                alt={quote.to.token}
                className="w-10 h-10 rounded-full"
              />
            ) : (
              <div className="w-10 h-10 rounded-full flex items-center justify-center">
                {quote.to.token.slice(0, 2)}
              </div>
            )}
            <div>
              <div className="font-bold flex items-center gap-2">
                {quote.to.token}
                <img
                  src={`https://dd.dexscreener.com/ds-data/chains/${toChain}.png`}
                  alt={toChain}
                  className="w-4 h-4 rounded-full"
                />
              </div>
              <div className="text-sm">
                {formatAmount(outputAmount, outputTokenDecimals)}
              </div>
              <div className="text-xs">
                {quote.to.address.slice(0, 6)}...
                {quote.to.address.slice(-4)}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Transaction Details */}
      <div className="mt-3 pt-3 border-t border-[#2D2D2D]">
        <div className="flex justify-between items-start">
          <div>
            <div className="text-sm text-white">
              Est. Time: {quote.execution_time_seconds}s
            </div>
            <div className="text-sm text-white">
              Slippage: {quote.slippage_percent}%
            </div>
          </div>
          <div className="text-right">
            <div className="text-sm text-white">Costs:</div>
            {Object.entries(quote.costs).map(([token, amount]) => {
              const decimals = getTokenDecimals(token, quote);
              return (
                <div key={token} className="text-sm text-white">
                  {formatAmount(amount, decimals)} {token}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};

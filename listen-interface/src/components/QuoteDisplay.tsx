import { useToken } from "../hooks/useToken";
import { formatAmount, imageMap } from "../hooks/util";
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
    <div className="border border-purple-500/30 rounded-lg p-4 bg-black/40 backdrop-blur-sm">
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
              <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center text-purple-300">
                {quote.from.token.slice(0, 2)}
              </div>
            )}
            <div>
              <div className="font-bold text-purple-100">
                {quote.from.token}
              </div>
              <div className="text-sm text-purple-300">
                {formatAmount(inputAmount, inputTokenDecimals)}
              </div>
              <div className="text-xs text-purple-300/70">
                {quote.from.address.slice(0, 6)}...
                {quote.from.address.slice(-4)}
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
                alt={quote.to.token}
                className="w-10 h-10 rounded-full"
              />
            ) : (
              <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center text-purple-300">
                {quote.to.token.slice(0, 2)}
              </div>
            )}
            <div>
              <div className="font-bold text-purple-100">{quote.to.token}</div>
              <div className="text-sm text-purple-300">
                {formatAmount(outputAmount, outputTokenDecimals)}
              </div>
              <div className="text-xs text-purple-300/70">
                {quote.to.address.slice(0, 6)}...
                {quote.to.address.slice(-4)}
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
              Est. Time: {quote.execution_time_seconds}s
            </div>
            <div className="text-sm text-purple-300">
              Slippage: {quote.slippage_percent}%
            </div>
          </div>
          <div className="text-right">
            <div className="text-sm text-purple-300">Costs:</div>
            {Object.entries(quote.costs).map(([token, amount]) => {
              const decimals = getTokenDecimals(token, quote);
              return (
                <div key={token} className="text-sm text-purple-300">
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

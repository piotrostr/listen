import { useEffect, useState } from "react";
import { FaTimes } from "react-icons/fa";
import { usePipelineExecution } from "../hooks/usePipelineExecution";
import { useSolBalance } from "../hooks/useSolBalance";

interface BuySellModalProps {
  isOpen: boolean;
  onClose: () => void;
  action: "buy" | "sell";
  asset: {
    address: string;
    name: string;
    symbol: string;
    amount: number;
    logoURI?: string;
    price: number;
    decimals: number;
  };
}

export function BuySellModal({
  isOpen,
  onClose,
  action,
  asset,
}: BuySellModalProps) {
  const [percentage, setPercentage] = useState(50);
  const { data: solBalance, refetch: refetchSolBalance } = useSolBalance();
  const { isExecuting, quickBuyToken, sellTokenForSol } =
    usePipelineExecution();

  // Always refetch SOL balance when modal is open
  useEffect(() => {
    if (isOpen) {
      refetchSolBalance();
    }
  }, [isOpen, refetchSolBalance]);

  // Prevent body scrolling when modal is open
  useEffect(() => {
    if (isOpen) {
      // Store original body overflow and padding
      const originalStyle = window.getComputedStyle(document.body);
      const originalOverflow = originalStyle.overflow;
      const originalPaddingRight = originalStyle.paddingRight;

      // Get the width of the scrollbar
      const scrollbarWidth =
        window.innerWidth - document.documentElement.clientWidth;

      // Apply styles to prevent scrolling and maintain layout
      document.body.style.overflow = "hidden";
      document.body.style.paddingRight = `${scrollbarWidth}px`;

      // Cleanup function to restore original styles
      return () => {
        document.body.style.overflow = originalOverflow;
        document.body.style.paddingRight = originalPaddingRight;
      };
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const calculateAmount = () => {
    if (action === "buy") {
      // Calculate SOL amount based on percentage of max available SOL (with safety margin)
      const maxSol = (solBalance || 0) * 0.95; // 95% max to leave room for fees
      return (maxSol * percentage) / 100;
    } else {
      // Calculate token amount based on percentage of holdings
      return (asset.amount * percentage) / 100;
    }
  };

  const formattedAmount = calculateAmount().toFixed(
    action === "buy" ? 2 : asset.decimals > 6 ? 6 : asset.decimals
  );

  const handleSubmit = async () => {
    const amount = calculateAmount();

    if (action === "buy") {
      await quickBuyToken(asset.address, amount, {
        onSuccess: onClose,
      });
    } else {
      await sellTokenForSol(asset.address, amount, asset.decimals, asset.name, {
        onSuccess: onClose,
      });
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
      <div className="relative w-full lg:max-w-md max-w-sm p-6 bg-black/80 border border-purple-500/30 rounded-lg shadow-xl max-h-[90vh] overflow-y-auto my-4">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-purple-300 hover:text-purple-100"
        >
          <FaTimes />
        </button>

        <h2 className="text-xl font-bold mb-4 text-purple-100">
          {action === "buy" ? "Buy" : "Sell"} {asset.symbol}
        </h2>

        <div className="flex items-center mb-4">
          {asset.logoURI && (
            <img
              src={asset.logoURI}
              alt={asset.symbol}
              className="w-10 h-10 rounded-full mr-3"
            />
          )}
          <div>
            <div className="font-bold text-purple-100">{asset.name}</div>
            <div className="text-sm text-purple-300">
              ${asset.price.toFixed(6)}
            </div>
          </div>
        </div>

        <div className="mb-6">
          <div className="flex justify-between mb-2">
            <span className="text-purple-300 text-sm">
              Amount ({percentage}%)
            </span>
            <span className="text-purple-300 text-sm">
              {action === "buy"
                ? `${formattedAmount} SOL ($${(calculateAmount() * (action === "buy" ? 1 : asset.price)).toFixed(2)})`
                : `${formattedAmount} ${asset.symbol} ($${(calculateAmount() * asset.price).toFixed(2)})`}
            </span>
          </div>

          <input
            type="range"
            min="1"
            max="100"
            value={percentage}
            onChange={(e) => setPercentage(parseInt(e.target.value))}
            className="w-full h-2 bg-purple-500/20 rounded-lg appearance-none cursor-pointer"
          />

          <div className="flex justify-between mt-1">
            <span className="text-purple-300 text-xs">1%</span>
            <span className="text-purple-300 text-xs">50%</span>
            <span className="text-purple-300 text-xs">100%</span>
          </div>
        </div>

        <div className="mb-4 p-3 bg-purple-500/10 rounded-lg">
          <div className="flex justify-between text-sm">
            <span className="text-purple-300">Available:</span>
            <span className="text-purple-100">
              {action === "buy"
                ? `${(solBalance || 0).toFixed(4)} SOL`
                : `${asset.amount.toFixed(asset.decimals > 6 ? 6 : asset.decimals)} ${asset.symbol}`}
            </span>
          </div>
        </div>

        <button
          onClick={handleSubmit}
          disabled={isExecuting}
          className={`w-full py-2 rounded-lg text-white font-medium transition-colors ${
            action === "buy"
              ? "bg-green-500/70 hover:bg-green-500"
              : "bg-red-500/70 hover:bg-red-500"
          } ${isExecuting ? "opacity-70 cursor-not-allowed" : ""}`}
        >
          {isExecuting
            ? "Processing..."
            : `${action === "buy" ? "Buy" : "Sell"} ${asset.symbol}`}
        </button>
      </div>
    </div>
  );
}

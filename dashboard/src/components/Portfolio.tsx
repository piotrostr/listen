import { useState } from "react";

const mockPortfolioData = [
  {
    id: 1,
    name: "Solana",
    symbol: "SOL",
    price: 101.23,
    change: 5.67,
    amount: 12.5,
  },
  {
    id: 2,
    name: "Fartcoin",
    symbol: "Fartcoin",
    price: 0.0042,
    change: -2.34,
    amount: 10000,
  },
  {
    id: 3,
    name: "Pengu",
    symbol: "PENGU",
    price: 0.89,
    change: 12.45,
    amount: 500,
  },
];

export function Portfolio() {
  const [assets] = useState(mockPortfolioData);

  return (
    <div className="h-[70vh] font-mono">
      <div className="h-full border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm overflow-hidden flex flex-col">
        <h2 className="text-xl font-bold p-4">Portfolio</h2>

        <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
          <div className="p-4 pt-0 space-y-4">
            {assets.map((asset) => (
              <div
                key={asset.id}
                className="border border-purple-500/30 rounded-lg p-3 hover:bg-purple-900/20 transition-colors"
              >
                <div className="flex justify-between items-start mb-2">
                  <div>
                    <h3 className="font-bold">{asset.name}</h3>
                    <p className="text-sm text-gray-400">{asset.symbol}</p>
                  </div>
                  <div className="text-right">
                    <p className="font-bold">${asset.price.toLocaleString()}</p>
                    <p
                      className={`text-sm ${
                        asset.change >= 0 ? "text-green-400" : "text-red-400"
                      }`}
                    >
                      {asset.change >= 0 ? "+" : ""}
                      {asset.change}%
                    </p>
                  </div>
                </div>
                <div className="text-sm text-gray-400">
                  Amount: {asset.amount} {asset.symbol}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

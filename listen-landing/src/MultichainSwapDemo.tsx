import React, { useEffect, useState } from "react";
import "./MultichainSwapDemo.css";

const MultichainSwapDemo: React.FC = () => {
  const [visibleMessages, setVisibleMessages] = useState<number>(0);
  const [approved, _] = useState<boolean | null>(null);

  const messages = [
    {
      type: "user",
      content:
        "lets make an order to swap 100 USD on sol into PEPE, BONK and BRETT",
    },
    {
      type: "assistant",
      content: (
        <>
          <p className="my-2">
            I'll help you create a pipeline to swap your USDC into PEPE, BONK,
            and BRETT tokens. Let me first note that:
          </p>
          <ol className="list-decimal pl-6 my-2">
            <li className="my-1">
              PEPE is not available on Solana, we'll need to bridge to Ethereum
              for that
            </li>
            <li className="my-1">BONK is on Solana</li>
            <li className="my-1">BRETT is on Base chain</li>
          </ol>
          <p className="my-2">I'll create a pipeline that will:</p>
          <ol className="list-decimal pl-6 my-2">
            <li className="my-1">
              Split your USDC into three parts (~33.33 USDC each)
            </li>
            <li className="my-1">Swap to each token respectively</li>
          </ol>
          <p className="my-2">Here's the pipeline:</p>
        </>
      ),
    },
    {
      type: "pipeline",
      content: (
        <div className="space-y-4">
          <PipelineStep
            number={1}
            fromToken={{
              symbol: "USDC",
              chain: "solana",
              amount: "33.33333",
              address: "EPjF...Dt1v",
              logoUrl:
                "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/logo.png",
            }}
            toToken={{
              symbol: "Bonk",
              chain: "solana",
              address: "DezX...B263",
              logoUrl:
                "https://arweave.net/hQiPZOsRZXGXBJd_82PhVdlM_hACsT_q6wqwf5cSY7I?ext=png",
            }}
          />

          <PipelineStep
            number={2}
            fromToken={{
              symbol: "USDC",
              chain: "solana",
              amount: "33.33333",
              address: "EPjF...Dt1v",
              logoUrl:
                "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/logo.png",
            }}
            toToken={{
              symbol: "PEPE",
              chain: "ethereum",
              address: "0x69...1933",
              logoUrl:
                "https://strapi.jumper.exchange/uploads/pepe_9f618b02d1.png",
            }}
          />

          <PipelineStep
            number={3}
            fromToken={{
              symbol: "USDC",
              chain: "solana",
              amount: "33.33333",
              address: "EPjF...Dt1v",
              logoUrl:
                "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/ethereum/assets/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/logo.png",
            }}
            toToken={{
              symbol: "BRETT",
              chain: "base",
              address: "0x53...42E4",
              logoUrl:
                "https://strapi.jumper.exchange/uploads/brett_ca2d328cc8.jpeg",
            }}
          />
        </div>
      ),
    },
  ];

  useEffect(() => {
    if (visibleMessages < messages.length) {
      const timer = setTimeout(() => {
        setVisibleMessages((prev) => prev + 1);
      }, 1000); // Show a new message every second

      return () => clearTimeout(timer);
    }
  }, [visibleMessages, messages.length]);

  return (
    <div className="flex-grow overflow-y-auto max-h-[80vh] pb-4 space-y-4 scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
      <div className="p-3">
        {/* Visible messages */}
        {messages.slice(0, visibleMessages).map((message, index) => (
          <div key={index} className="mb-4">
            {message.type === "user" && (
              <div className="bg-purple-900/20 text-purple-300 rounded-lg px-3 py-2 my-2 backdrop-blur-sm border border-opacity-20 text-sm border-purple-500">
                <div className="markdown-content">
                  <p className="my-1">{message.content}</p>
                </div>
              </div>
            )}

            {message.type === "assistant" && (
              <div className="bg-blue-900/20 text-blue-300 rounded-lg px-3 py-2 my-2 backdrop-blur-sm border border-opacity-20 text-sm border-blue-500">
                <div className="markdown-content">{message.content}</div>
              </div>
            )}

            {message.type === "pipeline" && (
              <div className="my-3 border-b border-purple-500/30 pb-3">
                {message.content}
              </div>
            )}
          </div>
        ))}

        {/* Hidden placeholder to reserve space */}
        <div className="invisible" aria-hidden="true">
          {messages.slice(visibleMessages).map((message, index) => (
            <div key={index} className="mb-4">
              {message.type === "user" && (
                <div className="bg-purple-900/20 text-purple-300 rounded-lg px-3 py-2 my-2 backdrop-blur-sm border border-opacity-20 text-sm border-purple-500">
                  <div className="markdown-content">
                    <p className="my-1">{message.content}</p>
                  </div>
                </div>
              )}

              {message.type === "assistant" && (
                <div className="bg-blue-900/20 text-blue-300 rounded-lg px-3 py-2 my-2 backdrop-blur-sm border border-opacity-20 text-sm border-blue-500">
                  <div className="markdown-content">{message.content}</div>
                </div>
              )}

              {message.type === "pipeline" && (
                <div className="my-3 border-b border-purple-500/30 pb-3">
                  {message.content}
                </div>
              )}
            </div>
          ))}
        </div>

        {approved === true && (
          <div className="bg-green-900/20 text-green-300 rounded-lg px-3 py-2 my-2 backdrop-blur-sm border border-opacity-20 text-sm border-green-500">
            <div className="markdown-content">
              <p className="my-1">
                Pipeline approved! Executing transactions...
              </p>
            </div>
          </div>
        )}

        {approved === false && (
          <div className="bg-red-900/20 text-red-300 rounded-lg px-3 py-2 my-2 backdrop-blur-sm border border-opacity-20 text-sm border-red-500">
            <div className="markdown-content">
              <p className="my-1">
                Pipeline rejected. No transactions will be executed.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

interface TokenInfo {
  symbol: string;
  chain: string;
  amount?: string;
  address: string;
  logoUrl: string;
}

interface PipelineStepProps {
  number: number;
  fromToken: TokenInfo;
  toToken: TokenInfo;
}

const PipelineStep: React.FC<PipelineStepProps> = ({
  number,
  fromToken,
  toToken,
}) => {
  const getChainIcon = (chain: string) => {
    const chainIcons: Record<string, string> = {
      solana: "https://dd.dexscreener.com/ds-data/chains/solana.png",
      ethereum: "https://dd.dexscreener.com/ds-data/chains/ethereum.png",
      base: "https://dd.dexscreener.com/ds-data/chains/base.png",
    };

    return chainIcons[chain] || "";
  };

  return (
    <div className="border border-purple-500/30 rounded-lg p-3 bg-black/40 backdrop-blur-sm">
      <div className="flex items-center gap-2 flex-wrap sm:flex-nowrap">
        <div className="text-xs text-purple-300 w-6 text-center">{number}</div>

        {/* From Token */}
        <div className="flex-1 min-w-[120px]">
          <div className="flex items-center gap-2">
            <div className="flex flex-col">
              <img
                src={fromToken.logoUrl}
                alt={fromToken.symbol}
                className="w-6 h-6 rounded-full"
              />
            </div>
            <div>
              <div className="flex items-center gap-1">
                <div className="font-bold text-purple-100 text-sm">
                  {fromToken.symbol}
                </div>
                <img
                  src={getChainIcon(fromToken.chain)}
                  alt={fromToken.chain}
                  className="w-3 h-3 rounded-full"
                />
              </div>
              {fromToken.amount && (
                <div className="text-xs text-purple-300">
                  Amount: {fromToken.amount}
                </div>
              )}
              <div className="text-xs text-gray-400 truncate max-w-[120px]">
                {fromToken.address}
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
            strokeWidth="1.5"
            stroke="currentColor"
            className="w-5 h-5"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"
            ></path>
          </svg>
        </div>

        {/* To Token */}
        <div className="flex-1 min-w-[120px]">
          <div className="flex items-center gap-2">
            <div className="flex flex-col">
              <img
                src={toToken.logoUrl}
                alt={toToken.symbol}
                className="w-6 h-6 rounded-full"
              />
            </div>
            <div>
              <div className="flex items-center gap-1">
                <div className="font-bold text-purple-100 text-sm">
                  {toToken.symbol}
                </div>
                <img
                  src={getChainIcon(toToken.chain)}
                  alt={toToken.chain}
                  className="w-3 h-3 rounded-full"
                />
              </div>
              <div className="text-xs text-gray-400 truncate max-w-[120px]">
                {toToken.address}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="mt-2 pt-2 border-t border-purple-500/30">
        <div className="text-xs text-purple-300">Conditions:</div>
        <div className="mt-1 text-xs text-purple-200">Execute immediately</div>
      </div>
    </div>
  );
};

export default MultichainSwapDemo;

import { z } from "zod";
import { CandlestickDataSchema } from "../hooks/types";
import { DexScreenerResponseSchema } from "../types/dexscreener";
import { ToolResult } from "../types/message";
import { TokenMetadataSchema } from "../types/metadata";
import {
  JupiterQuoteResponseSchema,
  QuoteResponseSchema,
} from "../types/quote";
import { TweetSchema } from "../types/x";
import { SolanaBalance, SplTokenBalance } from "./Balances";
import { InnerChart } from "./Chart";
import { ChatMessage } from "./ChatMessage";
import { DexscreenerDisplay } from "./DexscreenerDisplay";
import { FetchXPostDisplay } from "./FetchXPostDisplay";
import { JupiterQuoteDisplay } from "./JupiterQuoteDisplay";
import { TransactionLink } from "./PipelineStepContainer";
import { QuoteDisplay } from "./QuoteDisplay";
import { RawTokenMetadataDisplay } from "./RawTokenMetadataDisplay";
import { ToolOutputDisplay } from "./ToolOutputDisplay";
import { TopTokensDisplay, TopTokensResponseSchema } from "./TopTokensDisplay";

const SplTokenBalanceSchema = z.tuple([z.string(), z.number(), z.string()]);

export const ToolMessage = ({ toolOutput }: { toolOutput: ToolResult }) => {
  if (toolOutput.name === "get_spl_token_balance") {
    try {
      const parsed = SplTokenBalanceSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="p-3">
          <SplTokenBalance
            amount={parsed[0]}
            decimals={parsed[1]}
            mint={parsed[2]}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse spl token balance:", e);
    }
  }

  if (toolOutput.name === "fetch_x_post") {
    try {
      const parsed = TweetSchema.parse(JSON.parse(toolOutput.result));
      return <FetchXPostDisplay tweet={parsed} />;
    } catch (e) {
      console.error("Failed to parse tweet:", e);
      return <ChatMessage message={toolOutput.result} direction="agent" />;
    }
  }

  if (
    toolOutput.name === "research_x_profile" ||
    toolOutput.name === "search_tweets"
  ) {
    try {
      const message = JSON.parse(toolOutput.result);
      return (
        <div className="text-gray-400">
          <ChatMessage message={message} direction="agent" />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse tweet:", e);
      return <ChatMessage message={toolOutput.result} direction="agent" />;
    }
  }

  if (toolOutput.name === "fetch_token_metadata") {
    try {
      const parsed = TokenMetadataSchema.parse(JSON.parse(toolOutput.result));
      return <RawTokenMetadataDisplay metadata={parsed} />;
    } catch (e) {
      console.error("Failed to parse token metadata:", e);
    }
  }

  if (toolOutput.name === "get_sol_balance") {
    try {
      const parsedLamports = parseInt(toolOutput.result);
      const solanaBalance = parsedLamports / 10 ** 9;
      return (
        <div className="p-3">
          <SolanaBalance solanaBalance={solanaBalance} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse solana balance:", e);
      return <div>Error parsing solana balance</div>;
    }
  }
  // If it's a dexscreener response, parse and display it
  if (toolOutput.name === "search_on_dex_screener") {
    try {
      const parsed = DexScreenerResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <DexscreenerDisplay pairs={parsed.pairs} />;
    } catch (e) {
      console.error("Failed to parse dexscreener response:", e);
      return <div>Error parsing DexScreener data</div>;
    }
  }

  if (toolOutput.name === "fetch_candlesticks") {
    try {
      const parsed = CandlestickDataSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="h-[300px]">
          <InnerChart data={parsed} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse candlestick response:", e);
    }
  }

  if (toolOutput.name === "fetch_top_tokens") {
    try {
      const parsed = TopTokensResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <TopTokensDisplay tokens={parsed} />;
    } catch (e) {
      console.error("Failed to parse top tokens response:", e);
    }
  }
  if (toolOutput.name === "swap") {
    try {
      // TODO standardize this output, not just string but { status: string, transactionHash: string }
      return (
        <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500 overflow-hidden">
          <div className="mb-2 overflow-hidden">
            <TransactionLink
              status={"Completed"}
              transactionHash={JSON.parse(toolOutput.result)}
              error={null}
            />
          </div>
        </div>
      );
    } catch (e) {
      console.error("Failed to parse swap response:", e);
      return (
        <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500 overflow-hidden">
          <div className="mb-2 overflow-hidden">
            <TransactionLink
              status={"Failed"}
              transactionHash={null}
              error={toolOutput.result}
            />
          </div>
        </div>
      );
    }
  }

  if (toolOutput.name === "get_quote") {
    try {
      // Clean up the result string in case it has extra escaping
      let resultData = toolOutput.result;

      // Sometimes quotes get double-stringified, check for that
      if (resultData.startsWith('"') && resultData.endsWith('"')) {
        try {
          // Try to parse once to remove outer quotes if double-stringified
          resultData = JSON.parse(resultData);
        } catch (e) {
          // If this fails, keep the original string
          resultData = toolOutput.result;
        }
      }

      try {
        // Parse the data to an object
        const parsedData =
          typeof resultData === "string" ? JSON.parse(resultData) : resultData;

        // First try Jupiter quote schema
        try {
          const jupiterQuote = JupiterQuoteResponseSchema.parse(parsedData);
          return <JupiterQuoteDisplay quote={jupiterQuote} />;
        } catch (jupiterError) {
          console.error("Jupiter quote validation failed:", jupiterError);

          // Then try regular quote schema
          try {
            const quote = QuoteResponseSchema.parse(parsedData);
            return <QuoteDisplay quote={quote} />;
          } catch (quoteError) {
            console.error("Regular quote validation failed:", quoteError);
            throw new Error("Failed to validate quote with either schema");
          }
        }
      } catch (parseError) {
        console.error("JSON parse error:", parseError);
        throw parseError;
      }
    } catch (e) {
      console.error("Quote processing failed:", e);

      return (
        <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
          <p className="text-red-400 break-words">
            Failed to parse quote data:{" "}
            {e instanceof Error ? e.message : "Unknown error"}
          </p>
          <details>
            <summary className="cursor-pointer text-sm">
              View raw quote data
            </summary>
            <pre className="text-xs mt-2 overflow-x-auto p-2 bg-gray-800 rounded break-words whitespace-pre-wrap">
              {typeof toolOutput.result === "string"
                ? toolOutput.result
                : JSON.stringify(toolOutput.result, null, 2)}
            </pre>
          </details>
        </div>
      );
    }
  }
  // Default tool output display
  return (
    <div className="text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500 overflow-hidden">
      {toolOutput.name}
      <ToolOutputDisplay toolOutput={toolOutput} />
    </div>
  );
};

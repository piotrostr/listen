import ReactMarkdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import { CandlestickDataSchema, type ToolOutput } from "../hooks/types";
import { renderAddressOrTx } from "../hooks/util";
import { DexScreenerResponseSchema } from "../types/dexscreener";
import {
  JupiterQuoteResponseSchema,
  QuoteResponseSchema,
} from "../types/quote";
import { InnerChart } from "./Chart";
import { DexscreenerDisplay } from "./DexscreenerDisplay";
import { JupiterQuoteDisplay } from "./JupiterQuoteDisplay";
import { TransactionLink } from "./PipelineStepContainer";
import { QuoteDisplay } from "./QuoteDisplay";
import { ToolOutputDisplay } from "./ToolOutputDisplay";
import { TopTokensDisplay, TopTokensResponseSchema } from "./TopTokensDisplay";

export const ToolMessage = ({ toolOutput }: { toolOutput: ToolOutput }) => {
  // If it's a dexscreener response, parse and display it
  if (toolOutput.name === "search_on_dex_screener") {
    try {
      const parsed = DexScreenerResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return (
        <div className="bg-blue-900/20 text-blue-300 rounded-lg px-2 py-1 lg:px-4 lg:py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
          <DexscreenerDisplay pairs={parsed.pairs} />
        </div>
      );
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
    <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500 overflow-hidden">
      {toolOutput.name}
      <ToolOutputDisplay toolOutput={toolOutput} />
    </div>
  );
};

const sanitizeOutput = (message: string) => {
  const isProd = process.env.NODE_ENV === "production";
  if (isProd && message.includes("EOF while parsing an object")) {
    return null;
  }
  return message;
};

export const ChatMessage = ({
  message,
  direction,
}: {
  message: string;
  direction: "incoming" | "outgoing";
}) => {
  // Process the message to identify addresses and transactions
  const embeddedMessage = renderAddressOrTx(message);
  const sanitizedMessage = sanitizeOutput(embeddedMessage);

  if (!sanitizedMessage) {
    return null;
  }

  return (
    <div
      className={`
        ${direction === "incoming" ? "bg-blue-900/20 text-blue-300" : "bg-purple-900/20 text-purple-300"}
        rounded-lg px-4 py-2 my-2 backdrop-blur-sm
        border border-opacity-20
        lg:text-md text-sm
        ${direction === "incoming" ? "border-blue-500" : "border-purple-500"}
        break-words word-break-all max-w-full overflow-hidden
      `}
      style={{
        wordBreak: "break-word",
        overflowWrap: "break-word",
      }}
    >
      <ReactMarkdown
        className="markdown-content"
        components={{
          p: ({ children, ...props }) => (
            <p
              className="my-2"
              style={{
                wordBreak: "break-word",
                overflowWrap: "break-word",
              }}
              {...props}
            >
              {children}
            </p>
          ),
          h1: ({ ...props }) => (
            <h1 className="text-xl font-bold my-3" {...props} />
          ),
          h2: ({ ...props }) => (
            <h2 className="text-lg font-bold my-3" {...props} />
          ),
          h3: ({ ...props }) => (
            <h3 className="text-md font-bold my-2" {...props} />
          ),
          ul: ({ ...props }) => (
            <ul className="list-disc pl-6 my-2" {...props} />
          ),
          ol: ({ ...props }) => (
            <ol className="list-decimal pl-6 my-2" {...props} />
          ),
          li: ({ children, ...props }) => (
            <li className="my-1" {...props}>
              {children}
            </li>
          ),
          a: ({ ...props }) => (
            <a
              className="text-blue-400 underline"
              style={{
                wordBreak: "break-all",
                display: "inline-block",
                maxWidth: "100%",
              }}
              {...props}
            />
          ),
          blockquote: ({ children, ...props }) => (
            <blockquote
              className="border-l-4 border-gray-500 pl-4 my-2 italic overflow-hidden"
              {...props}
            >
              {children}
            </blockquote>
          ),
          code: ({ ...props }) => (
            <code
              className="block bg-gray-800 p-2 rounded my-2 overflow-x-auto text-sm"
              style={{
                wordBreak: "break-all",
                whiteSpace: "pre-wrap",
              }}
              {...props}
            />
          ),
          pre: ({ ...props }) => (
            <pre
              className="bg-gray-800 p-3 rounded my-3 overflow-x-auto"
              style={{
                wordBreak: "break-word",
                whiteSpace: "pre-wrap",
                maxWidth: "100%",
              }}
              {...props}
            />
          ),
          table: ({ ...props }) => (
            <table className="border-collapse my-3 w-full" {...props} />
          ),
          th: ({ ...props }) => (
            <th
              className="border border-gray-600 px-2 py-1 bg-gray-800"
              {...props}
            />
          ),
          td: ({ children, ...props }) => (
            <td className="border border-gray-600 px-2 py-1" {...props}>
              {children}
            </td>
          ),
          hr: ({ ...props }) => (
            <hr className="my-4 border-gray-600" {...props} />
          ),
        }}
        rehypePlugins={[rehypeRaw]}
      >
        {embeddedMessage}
      </ReactMarkdown>
    </div>
  );
};

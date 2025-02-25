import ReactMarkdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import { type ToolOutput } from "../hooks/types";
import { renderAddressOrTx } from "../hooks/util";
import { DexScreenerResponseSchema } from "../types/dexscreener";
import { QuoteResponseSchema } from "../types/quote";
import { DexscreenerDisplay } from "./DexscreenerDisplay";
import { TransactionLink } from "./PipelineStepContainer";
import { QuoteDisplay } from "./QuoteDisplay";
import { ToolOutputDisplay } from "./ToolOutputDisplay";

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

  if (toolOutput.name === "swap") {
    try {
      // TODO standardize this output, not just string but { status: string, transactionHash: string }
      return (
        <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
          <div className="mb-2">
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
      return <div>Error parsing swap response</div>;
    }
  }

  if (toolOutput.name === "get_quote") {
    try {
      const quote = QuoteResponseSchema.parse(JSON.parse(toolOutput.result));
      return <QuoteDisplay quote={quote} />;
    } catch (e) {
      console.error("Failed to parse quote response:", e);
    }
  }
  // Default tool output display
  return (
    <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
      {toolOutput.name}
      <ToolOutputDisplay toolOutput={toolOutput} />
    </div>
  );
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

  return (
    <div
      className={`
        ${direction === "incoming" ? "bg-blue-900/20 text-blue-300" : "bg-purple-900/20 text-purple-300"}
        rounded-lg px-4 py-2 my-2 backdrop-blur-sm
        border border-opacity-20
        lg:text-md text-sm
        ${direction === "incoming" ? "border-blue-500" : "border-purple-500"}
      `}
    >
      <ReactMarkdown
        className="markdown-content"
        components={{
          p: ({ children, ...props }) => (
            <p className="my-2" {...props}>
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
            <a className="text-blue-400 underline" {...props} />
          ),
          blockquote: ({ children, ...props }) => (
            <blockquote
              className="border-l-4 border-gray-500 pl-4 my-2 italic"
              {...props}
            >
              {children}
            </blockquote>
          ),
          code: ({ ...props }) => (
            <code
              className="block bg-gray-800 p-2 rounded my-2 overflow-x-auto text-sm"
              {...props}
            />
          ),
          pre: ({ ...props }) => (
            <pre
              className="bg-gray-800 p-3 rounded my-3 overflow-x-auto"
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
        // Add this prop to allow HTML to be rendered within the markdown
        rehypePlugins={[rehypeRaw]}
      >
        {embeddedMessage}
      </ReactMarkdown>
    </div>
  );
};

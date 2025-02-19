import ReactMarkdown from "react-markdown";
import { ToolOutput } from "../hooks/useChat";
import { DexScreenerResponseSchema } from "../types/dexscreener";
import { DexscreenerDisplay } from "./DexscreenerDisplay";
import { ToolOutputDisplay } from "./ToolOutputDisplay";

export const ToolMessage = ({ toolOutput }: { toolOutput: ToolOutput }) => {
  // If it's a dexscreener response, parse and display it
  if (toolOutput.name === "search_on_dex_screener") {
    try {
      const parsed = DexScreenerResponseSchema.parse(toolOutput.result);
      return (
        <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
          <DexscreenerDisplay pairs={parsed.pairs} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse dexscreener response:", e);
      return <div>Error parsing DexScreener data</div>;
    }
  }

  // Default tool output display
  return (
    <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
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
}) => (
  <div
    className={`
      ${direction === "incoming" ? "bg-blue-900/20 text-blue-300" : "bg-purple-900/20 text-purple-300"}
      rounded-lg px-4 py-2 my-2 backdrop-blur-sm
      border border-opacity-20
      ${direction === "incoming" ? "border-blue-500" : "border-purple-500"}
    `}
  >
    <ReactMarkdown>{message}</ReactMarkdown>
  </div>
);

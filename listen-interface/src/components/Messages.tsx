import { ToolOutput } from "../hooks/useChat";
import { ToolOutputDisplay } from "./ToolOutputDisplay";
import ReactMarkdown from "react-markdown";

export const ToolMessage = ({ toolOutput }: { toolOutput: ToolOutput }) => (
  <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
    <ToolOutputDisplay toolOutput={toolOutput} />
  </div>
);

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

import { ToolCall } from "../types/message";

export const ToolCallMessage = ({ toolCall }: { toolCall: ToolCall }) => {
  return (
    <div className="text-sm text-gray-400">Tool call: {toolCall.name}</div>
  );
};

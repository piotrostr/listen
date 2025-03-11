import { ToolCall } from "../types/message";

export const ToolCallMessage = ({ toolCall }: { toolCall: ToolCall }) => {
  return (
    <div className="flex flex-col gap-2">
      <div className="text-sm text-gray-500">Tool call: {toolCall.name}</div>
    </div>
  );
};

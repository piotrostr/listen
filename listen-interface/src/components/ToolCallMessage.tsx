import { useTranslation } from "react-i18next";
import { ToolCall } from "../types/message";

export const ToolCallMessage = ({ toolCall }: { toolCall: ToolCall }) => {
  const { t } = useTranslation();
  return (
    <div className="text-sm text-gray-400">
      {t(`tool_calls.${toolCall.name}`)}
    </div>
  );
};

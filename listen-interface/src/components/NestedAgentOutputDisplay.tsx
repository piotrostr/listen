import { renderAgentOutput } from "../parse-agent-output";
import { Markdown } from "./ChatMessage";

export const NestedAgentOutputDisplay = ({ content }: { content: string }) => {
  return (
    <div className="px-3 py-2 my-1 text-gray-500">
      <Markdown message={renderAgentOutput(content, false)} />
    </div>
  );
};

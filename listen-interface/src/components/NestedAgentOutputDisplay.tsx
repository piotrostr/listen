import { parseAgentOutput } from "../parse-agent-output";
import { Markdown } from "./ChatMessage";

export const NestedAgentOutputDisplay = ({ content }: { content: string }) => {
  // this will all be of nested output
  const parsed = parseAgentOutput(content);
  console.log("parsed", parsed);
  return (
    <div className="px-3 py-2 my-1 text-gray-500">
      {parsed.map((p, index) => {
        return <Markdown key={index} message={JSON.stringify(p)} />;
      })}
    </div>
  );
};

import ReactMarkdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import remarkGfm from "remark-gfm";
import { renderAddressOrTx } from "../lib/util";
import { renderAgentOutput } from "../parse-agent-output";

const sanitizeOutput = (message: string) => {
  const isProd = process.env.NODE_ENV === "production";

  // Handle null/undefined
  if (!message) return "";

  if (isProd && message.includes("EOF while parsing an object")) {
    return "";
  }

  return removeMarkdownTags(message);
};

const removeMarkdownTags = (message: string) => {
  return message.replace(/^```markdown\s*|\s*```$/g, "");
};

export const ChatMessage = ({
  message,
  direction,
}: {
  message: string;
  direction: "incoming" | "outgoing" | "agent";
}) => {
  if (message.includes("<content>")) {
    message = renderAgentOutput(message, true);
  }
  // First sanitize the message
  const sanitizedMessage = sanitizeOutput(message);

  if (!sanitizedMessage) {
    return null;
  }

  // Then process addresses and transactions
  const embeddedMessage = renderAddressOrTx(sanitizedMessage);

  return (
    <div
      className={`
        rounded-lg px-4 py-1 my-2 font-light
        break-words word-break-all overflow-hidden
        ${direction === "outgoing" ? "rounded-3xl bg-[#2f2f2f]/40 ml-auto" : "max-w-full"}
      `}
      style={{
        wordBreak: "break-word",
        overflowWrap: "break-word",
      }}
    >
      <Markdown message={embeddedMessage} />
    </div>
  );
};

export const Markdown = ({ message }: { message: string }) => {
  return (
    <ReactMarkdown
      className="markdown-content"
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[rehypeRaw]}
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
        strong: ({ children, ...props }) => (
          <strong style={{ fontWeight: 700 }} className="font-bold" {...props}>
            {children}
          </strong>
        ),
        b: ({ children, ...props }) => (
          <b style={{ fontWeight: 700 }} className="font-bold" {...props}>
            {children}
          </b>
        ),
        h1: ({ ...props }) => (
          <h1
            className="text-xl font-bold my-3"
            style={{ fontWeight: 700 }}
            {...props}
          />
        ),
        h2: ({ ...props }) => (
          <h2
            className="text-lg font-bold my-3"
            style={{ fontWeight: 700 }}
            {...props}
          />
        ),
        h3: ({ ...props }) => (
          <h3
            className="text-md font-bold my-2"
            style={{ fontWeight: 700 }}
            {...props}
          />
        ),
        ul: ({ ...props }) => (
          <ul className="list-disc pl-10 my-2" {...props} />
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
            className="bg-transparent rounded overflow-x-auto inline"
            style={{
              wordBreak: "normal",
              whiteSpace: "normal",
            }}
            {...props}
          />
        ),
        pre: ({ ...props }) => (
          <pre
            className="bg-transparent rounded overflow-x-auto"
            style={{
              wordBreak: "break-word",
              whiteSpace: "pre-wrap",
              maxWidth: "100%",
            }}
            {...props}
          />
        ),
        table: ({ ...props }) => (
          <div className="w-full overflow-x-auto">
            <table
              className="border-collapse my-3 w-full text-sm whitespace-nowrap md:whitespace-normal"
              {...props}
            />
          </div>
        ),
        th: ({ ...props }) => (
          <th
            className="border border-[#2D2D2D] px-4 py-2 bg-[#2d2d2d] font-bold text-left"
            {...props}
          />
        ),
        td: ({ children, ...props }) => (
          <td className="border border-[#2D2D2D] px-4 py-2" {...props}>
            {children}
          </td>
        ),
        hr: ({ ...props }) => (
          <hr className="my-4 border-[#2D2D2D]" {...props} />
        ),
        em: ({ children, ...props }) => (
          <em className="italic text-gray-300" {...props}>
            {children}
          </em>
        ),
      }}
    >
      {message}
    </ReactMarkdown>
  );
};

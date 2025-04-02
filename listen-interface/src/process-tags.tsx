import { ChatMessage } from "./components/ChatMessage";
import { FundWallet } from "./components/FundWallet";
import { PipelineDisplay } from "./components/Pipeline";
import { SolanaWalletCreation } from "./components/SolanaWalletCreation";
import { Message } from "./types/message";

// New function to process a message with all supported tags
export function processMessageWithAllTags(
  message: string,
  msg: Message
): JSX.Element {
  // Create a structure to track all tag positions
  type TagPosition = {
    tagName: string;
    startIndex: number;
    endIndex: number;
    content: string;
  };

  const tagPositions: TagPosition[] = [];

  // Find all tag positions for all supported tag types
  Object.keys(tagHandlers).forEach((tagName) => {
    const tagRegex = new RegExp(`<${tagName}>(.*?)<\\/${tagName}>`, "gs");
    let match;

    while ((match = tagRegex.exec(message)) !== null) {
      tagPositions.push({
        tagName,
        startIndex: match.index,
        endIndex: match.index + match[0].length,
        content: match[1],
      });
    }
  });

  // Sort tag positions by their start index to maintain order
  tagPositions.sort((a, b) => a.startIndex - b.startIndex);

  // If no tags were found, return the original message
  if (tagPositions.length === 0) {
    return <ChatMessage message={message} direction={msg.direction} />;
  }

  // Split the message into parts
  const result: JSX.Element[] = [];
  let lastIndex = 0;

  tagPositions.forEach((pos, index) => {
    // Add text before the tag if there is any
    if (pos.startIndex > lastIndex) {
      const textBefore = message.substring(lastIndex, pos.startIndex);
      if (textBefore.trim()) {
        result.push(
          <ChatMessage
            key={`text-${index}`}
            message={textBefore}
            direction={msg.direction}
          />
        );
      }
    }

    // Process the tag content
    const handler = tagHandlers[pos.tagName];
    if (handler) {
      const processedTag = handler.processTag(pos.content, index, msg);
      result.push(processedTag);
    }

    lastIndex = pos.endIndex;
  });

  // Add any remaining text after the last tag
  if (lastIndex < message.length) {
    const textAfter = message.substring(lastIndex);
    if (textAfter.trim()) {
      result.push(
        <ChatMessage
          key={`text-final`}
          message={textAfter}
          direction={msg.direction}
        />
      );
    }
  }

  return <div>{result}</div>;
}

// Type definitions for tag handlers
type TagHandler = {
  processTag: (content: string, index: number, msg: Message) => JSX.Element;
  wrapResults?: (results: JSX.Element[]) => JSX.Element;
};

// Registry of tag handlers
export const tagHandlers: Record<string, TagHandler> = {
  pipeline: {
    processTag: (content: string, index: number, msg: Message) => {
      try {
        const pipelineContent = content
          .trim()
          .replace(/\/\*[\s\S]*?\*\/|\/\/.*/g, ""); // Remove comments

        const pipeline = JSON.parse(pipelineContent);
        if (pipeline && pipeline.steps) {
          return (
            <div key={`pipeline-${index}`} className="my-4 pb-4">
              <PipelineDisplay pipeline={pipeline} />
            </div>
          );
        }
      } catch (e) {
        console.error(`Failed to parse pipeline JSON #${index + 1}:`, e);
        // If we can't parse the JSON, just render the raw content
        return (
          <ChatMessage
            key={`pipeline-error-${index}`}
            message={`<pipeline>${content}</pipeline>`}
            direction={msg.direction}
          />
        );
      }
      return <></>;
    },
    wrapResults: (results: JSX.Element[]) => (
      <div className="mb-6">{results}</div>
    ),
  },
  setup_solana_wallet: {
    processTag: (_content: string, index: number) => {
      return (
        <div key={`setup-solana-wallet-${index}`}>
          <SolanaWalletCreation error={null} />
        </div>
      );
    },
  },
  fund_solana_wallet: {
    processTag: (_content: string, index: number) => {
      return (
        <div key={`fund-solana-wallet-${index}`}>
          <FundWallet />
        </div>
      );
    },
  },
};

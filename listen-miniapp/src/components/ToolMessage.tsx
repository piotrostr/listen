import { ReactNode, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { BsClock } from "react-icons/bs";
import {
  FaChartLine,
  FaCheckCircle,
  FaExclamationTriangle,
  FaSearch,
} from "react-icons/fa";
import { FaImage, FaRobot, FaXTwitter } from "react-icons/fa6";
import { IoSwapHorizontal } from "react-icons/io5";
import { z } from "zod";
import {
  CandlestickDataSchema,
  PriceActionAnalysisResponseSchema,
  TokenSchema,
} from "../lib/types";
import { renderTimestamps } from "../lib/util";
import { DexScreenerResponseSchema } from "../types/dexscreener";
import {
  Message,
  RigToolCall,
  ToolCall,
  ToolCallSchema,
  ToolResult,
} from "../types/message";
import {
  GtTokenMetadataSchema,
  TokenMetadataRawSchema,
} from "../types/metadata";
import {
  JupiterQuoteResponseSchema,
  QuoteResponseSchema,
} from "../types/quote";
import { TweetSchema } from "../types/x";
import { EthereumBalance, SolanaBalance, SplTokenBalance } from "./Balances";
import {
  BubbleMapDisplay,
  TokenHolderAnalysisSchema,
} from "./BubbleMapDisplay";
import { Chart, InnerChart } from "./Chart";
import { ChatMessage } from "./ChatMessage";
import { DexscreenerDisplay } from "./DexscreenerDisplay";
import DropdownMessage from "./DropdownMessage";
import { Erc20Balance, Erc20BalanceSchema } from "./Erc20Balance";
import { EvmRawTokenMetadataDisplay } from "./EvmRawTokenMetadataDisplay";
import { FetchXPostDisplay } from "./FetchXPostDisplay";
import { GeckoTerminalChart } from "./GeckoTerminalChart";
import { JupiterQuoteDisplay } from "./JupiterQuoteDisplay";
import { TransactionLink } from "./PipelineStepContainer";
import { QuoteDisplay } from "./QuoteDisplay";
import { RawTokenMetadataDisplay } from "./RawTokenMetadataDisplay";
import { embedResearchAnchors } from "./ResearchOutput";
import { RiskAnalysisDisplay, RiskAnalysisSchema } from "./RiskDisplay";
import { TokenDisplay } from "./TokenDisplay";
import { TopTokensDisplay, TopTokensResponseSchema } from "./TopTokensDisplay";
import { TopicDisplay, TopicSchema } from "./TopicDisplay";

const SplTokenBalanceSchema = z.tuple([z.string(), z.number(), z.string()]);
const EthBalanceSchema = z.tuple([z.string(), z.number()]);

const formatError = (error: string) => {
  if (error.includes("Invalid param: could not find account")) {
    return "Account not found";
  }
  return error;
};

const parseAndCleanMessage = (input: string): string => {
  try {
    let parsed = input;

    // First, try to parse any JSON strings
    while (
      typeof parsed === "string" &&
      (parsed.startsWith("{") ||
        parsed.startsWith("[") ||
        parsed.startsWith('"') ||
        parsed.includes('\\\"') ||
        parsed.includes("\\\\"))
    ) {
      try {
        parsed = JSON.parse(parsed);
      } catch (e) {
        break;
      }
    }

    // If we have a string, clean up any remaining escape sequences
    if (typeof parsed === "string") {
      // Remove any remaining escaped newlines that should be actual newlines
      parsed = parsed.replace(/\\n/g, "\n");

      // Remove any remaining double backslashes
      parsed = parsed.replace(/\\\\/g, "\\");

      // Remove unnecessary escaping of quotes around markdown content
      parsed = parsed.replace(
        /\\?"((?:[^"\\]|\\.)*)\\?"/g,
        (match, content) => {
          // If content contains markdown symbols, remove the quotes
          if (
            content.includes("**") ||
            content.includes("*") ||
            content.includes("```") ||
            content.includes("#") ||
            content.startsWith("- ") ||
            content.includes("\n- ")
          ) {
            return content;
          }
          return match;
        }
      );
    }

    // If it's not a string, stringify it nicely
    const res =
      typeof parsed === "string" ? parsed : JSON.stringify(parsed, null, 2);
    return res;
  } catch (e) {
    console.error("[parsing error]:", e);
    return input;
  }
};

export const ToolMessage = ({
  toolOutput,
  messages,
  currentMessage,
  toolCallData,
}: {
  toolOutput: ToolResult;
  messages: Message[];
  currentMessage: Message;
  toolCallData?: RigToolCall | ToolCall | null;
}) => {
  const { t } = useTranslation();

  // Use provided toolCallData if available, otherwise search for the corresponding tool call
  const toolCallInfo = useMemo(() => {
    if (toolCallData) {
      // If toolCallData is provided (likely from ParToolResultMessage),
      // adapt it slightly to match the ToolCallSchema shape for consistency where needed.
      // Note: RigToolCall uses `arguments`, ToolCallSchema uses `params` (stringified JSON).
      if ("function" in toolCallData) {
        // Check if it's RigToolCall
        return {
          id: toolCallData.id,
          name: toolCallData.function.name,
          // Attempt to stringify arguments, handle potential errors
          params: (() => {
            try {
              return JSON.stringify(toolCallData.function.arguments);
            } catch (e) {
              console.error("Failed to stringify RigToolCall arguments:", e);
              return "{}"; // Default to empty object string on error
            }
          })(),
          // Store the original arguments object for direct access if needed
          _arguments: toolCallData.function.arguments,
        };
      } else {
        // Assume it's ToolCallSchema-like
        return toolCallData;
      }
    }
    if (!toolOutput.id && !toolOutput.name) return null;
    const currentIndex = messages.findIndex((m) => m.id === currentMessage.id);
    if (currentIndex === -1) return null;

    for (let i = currentIndex - 1; i >= 0; i--) {
      const message = messages[i];
      if (message.type === "ToolCall") {
        try {
          const toolCall = ToolCallSchema.parse(JSON.parse(message.message));
          if (toolCall.id === toolOutput.id) {
            return toolCall; // Return the found ToolCall
          }
        } catch (e) {
          console.error("Failed to parse tool call during search:", e);
        }
      }

      // Optimization: Stop searching if we hit an outgoing message or another result
      if (
        message.direction === "outgoing" ||
        message.type === "ToolResult" ||
        message.type === "ParToolResult"
      ) {
        break;
      }
    }

    return null;
  }, [
    toolCallData,
    messages,
    currentMessage.id,
    toolOutput.id,
    toolOutput.name,
  ]);

  if (toolOutput.name === "think") {
    return null;
  }

  if (toolOutput.name === "get_token") {
    try {
      const parsed = TokenSchema.safeParse(JSON.parse(toolOutput.result));
      if (parsed.success) {
        return <TokenDisplay token={parsed.data} />;
      }
    } catch (e) {
      console.error("Failed to parse token:", e);
    }
    return (
      <div className="text-gray-400">
        <ChatMessage message={toolOutput.result} direction="agent" />
      </div>
    );
  }

  if (toolOutput.name === "fetch_price_action_analysis_evm") {
    const params = useMemo(() => {
      if (!toolCallInfo) return null;
      try {
        // Use pre-parsed arguments if available (from RigToolCall)
        if ("_arguments" in toolCallInfo && toolCallInfo._arguments) {
          return toolCallInfo._arguments as Record<string, any>;
        }
        // Otherwise parse the params string (from ToolCall or adapted RigToolCall)
        // Check if params exists and is a string before parsing
        if (
          "params" in toolCallInfo &&
          typeof toolCallInfo.params === "string"
        ) {
          return JSON.parse(toolCallInfo.params);
        } else if ("params" in toolCallInfo) {
          // Log a warning if params exists but is not a string
          console.warn(
            "Tool call 'params' exists but is not a string:",
            toolCallInfo.params
          );
          return null; // Return null as we can't parse it
        }
        // Redundant else-if removed
        return null; // Return null if neither _arguments nor valid params string is found
      } catch (e) {
        console.error("Failed to parse tool call params:", e);
        return null;
      }
    }, [toolCallInfo]);

    console.debug(params);
    const pairAddress = params?.pair_address;
    const interval = params?.interval || "30s";
    const chainId = params?.chain_id;

    if (pairAddress) {
      let parsed = toolOutput.result;
      try {
        parsed = JSON.parse(toolOutput.result);
      } catch (e) {
        console.error("Failed to parse price action analysis:", e);
      }
      const withAggregates =
        PriceActionAnalysisResponseSchema.safeParse(parsed);
      const analysis = withAggregates.success
        ? withAggregates.data.analysis
        : parsed;
      return (
        <div className="mb-1">
          <div className="h-[350px] mb-3">
            <GeckoTerminalChart
              pairAddress={pairAddress}
              chainId={chainId}
              timeframe={interval}
            />
          </div>
          <DropdownMessage
            title={t("tool_messages.price_action_analysis")}
            message={renderTimestamps(analysis)}
            icon={<FaChartLine />}
          />
        </div>
      );
    }

    return (
      <div className="text-gray-400">
        <ChatMessage message={toolOutput.result} direction="agent" />
      </div>
    );
  }

  if (toolOutput.name === "fetch_price_action_analysis") {
    try {
      // Extract parameters using toolCallInfo
      const params = useMemo(() => {
        if (!toolCallInfo) return null;
        try {
          // Use pre-parsed arguments if available (from RigToolCall)
          if ("_arguments" in toolCallInfo && toolCallInfo._arguments) {
            return toolCallInfo._arguments as Record<string, any>;
          }
          // Otherwise parse the params string (from ToolCall or adapted RigToolCall)
          // Check if params exists and is a string before parsing
          if (
            "params" in toolCallInfo &&
            typeof toolCallInfo.params === "string"
          ) {
            return JSON.parse(toolCallInfo.params);
          } else if ("params" in toolCallInfo) {
            // Log a warning if params exists but is not a string
            console.warn(
              "Tool call 'params' exists but is not a string:",
              toolCallInfo.params
            );
            return null; // Return null as we can't parse it
          }
          // Redundant else-if removed
          return null; // Return null if neither _arguments nor valid params string is found
        } catch (e) {
          console.error("Failed to parse tool call params:", e);
          return null;
        }
      }, [toolCallInfo]);

      const mint = params?.mint;
      const interval = params?.interval || "30s";

      if (mint) {
        let parsed = toolOutput.result;
        try {
          parsed = JSON.parse(toolOutput.result);
        } catch (e) {
          console.error("Failed to parse price action analysis:", e);
        }
        const withAggregates =
          PriceActionAnalysisResponseSchema.safeParse(parsed);
        const analysis = withAggregates.success
          ? withAggregates.data.analysis
          : parsed;
        return (
          <div className="mb-1">
            <div className="h-[300px] mb-3">
              <Chart mint={mint} interval={interval} />
            </div>
            <DropdownMessage
              title={t("tool_messages.price_action_analysis")}
              message={renderTimestamps(analysis)}
              icon={<FaChartLine />}
            />
          </div>
        );
      }

      return (
        <div className="text-gray-400">
          <ChatMessage message={toolOutput.result} direction="agent" />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse price action analysis:", e);
      return (
        <div className="text-gray-400">
          <ChatMessage message={toolOutput.result} direction="agent" />
        </div>
      );
    }
  }

  if (toolOutput.name === "get_eth_balance") {
    try {
      const parsed = EthBalanceSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="p-3">
          <EthereumBalance ethereumBalance={parsed[0]} chainId={parsed[1]} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse eth balance:", e);
    }
  }

  if (toolOutput.name === "get_erc20_balance") {
    try {
      const parsed = Erc20BalanceSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="p-3">
          <Erc20Balance erc20Balance={parsed} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse erc20 balance:", e);
    }
  }

  if (toolOutput.name === "get_spl_token_balance") {
    try {
      const parsed = SplTokenBalanceSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="p-3">
          <SplTokenBalance
            amount={parsed[0]}
            decimals={parsed[1]}
            mint={parsed[2]}
          />
        </div>
      );
    } catch (e) {
      if (toolOutput.result.includes("data: Empty") && toolCallInfo) {
        try {
          // Type-safe extraction of parameters for error case
          let params: Record<string, any> | null = null;
          if (
            toolCallInfo &&
            "_arguments" in toolCallInfo &&
            toolCallInfo._arguments
          ) {
            params = toolCallInfo._arguments as Record<string, any>;
          } else if (
            toolCallInfo &&
            "params" in toolCallInfo &&
            typeof toolCallInfo.params === "string"
          ) {
            try {
              params = JSON.parse(toolCallInfo.params);
            } catch (parseError) {
              console.error(
                "Failed to parse params in error handler:",
                parseError
              );
            }
          }

          const mint = params?.mint;

          if (!mint) throw new Error("Mint not found in tool call info");

          return (
            <div className="p-3">
              <div className="flex items-center gap-1">
                <SplTokenBalance amount={"0"} decimals={0} mint={mint} />
                <div className="relative group">
                  <span className="text-orange-500 flex items-center gap-1 cursor-help">
                    <FaExclamationTriangle />
                  </span>
                  <div className="absolute left-0 bottom-full mb-2 hidden group-hover:block bg-black/90 text-orange-500 p-2 rounded shadow-lg z-10 max-w-xs break-words w-[200px]">
                    {formatError(toolOutput.result)}
                  </div>
                </div>
              </div>
            </div>
          );
        } catch (e) {
          console.error("Failed to parse tool call:", e);
        }
      }
      console.error("Failed to parse spl token balance:", e);
    }
  }

  if (toolOutput.name == "view_image") {
    const message = JSON.parse(toolOutput.result);
    return (
      <div className="p-3">
        <DropdownMessage
          title={t("tool_messages.view_image")}
          message={message}
          icon={<FaImage />}
        />
      </div>
    );
  }

  if (toolOutput.name === "fetch_x_post") {
    try {
      const parsed = TweetSchema.parse(JSON.parse(toolOutput.result));
      return <FetchXPostDisplay tweet={parsed} />;
    } catch (e) {
      if (toolOutput.result.includes("No tweet found")) {
        return (
          <div className="p-3">
            <div className="text-orange-500 flex items-center gap-1">
              <FaExclamationTriangle /> {t("tool_messages.no_tweet_found")}
            </div>
          </div>
        );
      }
      return <ChatMessage message={toolOutput.result} direction="agent" />;
    }
  }

  if (toolOutput.name === "research_x_profile") {
    try {
      const message = JSON.parse(toolOutput.result);
      const processedMessage = embedResearchAnchors(message);
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.research_x_profile")}
            message={processedMessage}
            icon={<FaXTwitter />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse tweet:", e);
      if (toolOutput.result.includes("Account suspended")) {
        return (
          <div className="p-3">
            <div className="text-orange-500 flex items-center gap-1">
              <FaExclamationTriangle /> {t("tool_messages.account_suspended")}
            </div>
          </div>
        );
      }
      if (toolOutput.result.includes("not found")) {
        return (
          <div className="p-3">
            <div className="text-orange-500 flex items-center gap-1">
              <FaExclamationTriangle /> {t("tool_messages.user_does_not_exist")}
            </div>
          </div>
        );
      }
      return <ChatMessage message={toolOutput.result} direction="agent" />;
    }
  }

  if (
    toolOutput.result.includes("ToolCallError") &&
    !toolOutput.name.includes("delegate")
  ) {
    return null;
    // return (
    //   <Tooltip.Provider>
    //     <Tooltip.Root delayDuration={100}>
    //       <Tooltip.Trigger asChild>
    //         <div className="text-red-400 flex items-center gap-1 p-3 text-sm cursor-help">
    //           <FaExclamationTriangle /> {t("tool_messages.tool_call_error")}
    //         </div>
    //       </Tooltip.Trigger>
    //       <Tooltip.Portal>
    //         <Tooltip.Content className="rounded-md bg-[#2d2d2d] px-4 py-2 text-sm text-white max-w-md break-words shadow-lg z-50">
    //           {toolOutput.result}
    //           <Tooltip.Arrow className="fill-[#2d2d2d]" />
    //         </Tooltip.Content>
    //       </Tooltip.Portal>
    //     </Tooltip.Root>
    //   </Tooltip.Provider>
    // );
  }

  if (toolOutput.name === "search_web") {
    try {
      const message = JSON.parse(toolOutput.result);
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.search_web")}
            message={message}
            icon={<FaSearch />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse web search:", e);
    }
  }
  if (toolOutput.name === "analyze_page_content") {
    try {
      const message = JSON.parse(toolOutput.result);
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.analyze_page_content")}
            message={message}
            icon={<FaSearch />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse page content:", e);
    }
  }

  if (toolOutput.name === "search_tweets") {
    try {
      const message = JSON.parse(toolOutput.result);
      const processedMessage = embedResearchAnchors(message);
      return (
        <div className="mb-1">
          <DropdownMessage
            title={t("tool_messages.search_tweets")}
            message={processedMessage}
            icon={<FaXTwitter />}
          />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse tweet:", e);
    }
  }

  if (toolOutput.name === "get_current_time") {
    try {
      const parsed = JSON.parse(toolOutput.result);
      return (
        <div className="text-blue-300 flex items-center gap-1 p-3 text-sm">
          <BsClock /> {new Date(parsed).toLocaleString()}
        </div>
      );
    } catch (e) {
      console.error("Failed to parse current time:", e);
    }
  }

  if (toolOutput.name === "create_advanced_order") {
    try {
      const parsed = JSON.parse(toolOutput.result);
      return (
        <div className="text-green-300 flex items-center gap-1 p-3 text-sm">
          <FaCheckCircle /> {parsed}
        </div>
      );
    } catch (e) {
      console.error("Failed to parse advanced order:", e);
    }
  }

  if (toolOutput.name === "analyze_risk") {
    try {
      const parsed = RiskAnalysisSchema.parse(JSON.parse(toolOutput.result));
      return <RiskAnalysisDisplay riskAnalysis={parsed} />;
    } catch (e) {
      console.error("Failed to parse risk analysis:", e);
    }
  }
  if (toolOutput.name === "analyze_holder_distribution") {
    try {
      const parsed = TokenHolderAnalysisSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <BubbleMapDisplay topHolderAnalysis={parsed} />;
    } catch (e) {
      console.error("Failed to parse holder distribution:", e);
    }
  }

  if (toolOutput.name === "analyze_sentiment") {
    try {
      const parsed = TopicSchema.parse(JSON.parse(toolOutput.result));
      return <TopicDisplay topic={parsed} />;
    } catch (e) {
      console.error("Failed to parse sentiment:", e);
    }
  }

  if (toolOutput.name === "fetch_top_tokens_by_category") {
    try {
      const parsed = TopTokensResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <TopTokensDisplay tokens={parsed} />;
    } catch (e) {
      console.error("Failed to parse top tokens response:", e);
    }
  }

  if (
    toolOutput.name === "delegate_to_research_agent" ||
    toolOutput.name === "delegate_to_chart_agent" ||
    toolOutput.name === "delegate_to_solana_trader_agent"
  ) {
    const icons = {
      delegate_to_research_agent: <FaRobot />,
      delegate_to_chart_agent: <FaChartLine />,
      delegate_to_solana_trader_agent: <IoSwapHorizontal />,
    };

    const escapedMessage = parseAndCleanMessage(toolOutput.result);

    return (
      <div className="text-gray-400">
        <DropdownMessage
          title={t(`tool_messages.${toolOutput.name}`)}
          message={escapedMessage}
          icon={icons[toolOutput.name]}
        />
      </div>
    );
  }

  if (toolOutput.name === "fetch_token_metadata") {
    try {
      const parsed = TokenMetadataRawSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <RawTokenMetadataDisplay metadata={parsed} />;
    } catch (e) {
      console.error("Failed to parse token metadata:", e);
    }
  }

  if (toolOutput.name === "fetch_token_metadata_evm") {
    try {
      const parsed = GtTokenMetadataSchema.parse(JSON.parse(toolOutput.result));
      return <EvmRawTokenMetadataDisplay metadata={parsed} />;
    } catch (e) {
      console.error("Failed to parse EVM token metadata:", e);
    }
  }

  if (toolOutput.name === "get_sol_balance") {
    try {
      const parsedLamports = parseInt(toolOutput.result);
      const solanaBalance = parsedLamports / 10 ** 9;
      return (
        <div className="p-3">
          <SolanaBalance solanaBalance={solanaBalance} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse solana balance:", e);
      return <div>Error parsing solana balance</div>;
    }
  }
  // If it's a dexscreener response, parse and display it
  if (toolOutput.name === "search_on_dex_screener") {
    try {
      const parsed = DexScreenerResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <DexscreenerDisplay pairs={parsed.pairs} />;
    } catch (e) {
      console.error("Failed to parse dexscreener response:", e);
      return <div>Error parsing DexScreener data</div>;
    }
  }

  if (toolOutput.name === "fetch_candlesticks") {
    try {
      const parsed = CandlestickDataSchema.parse(JSON.parse(toolOutput.result));
      return (
        <div className="h-[300px]">
          <InnerChart data={parsed} />
        </div>
      );
    } catch (e) {
      console.error("Failed to parse candlestick response:", e);
    }
  }

  if (
    toolOutput.name === "fetch_top_tokens" ||
    toolOutput.name === "fetch_top_tokens_by_chain_id"
  ) {
    try {
      const parsed = TopTokensResponseSchema.parse(
        JSON.parse(toolOutput.result)
      );
      return <TopTokensDisplay tokens={parsed} />;
    } catch (e) {
      console.error("Failed to parse top tokens response:", e);
    }
  }
  if (toolOutput.name === "swap") {
    const TxContainer = ({ children }: { children: ReactNode }) => (
      <div className="text-white px-4 py-3 my-2 backdrop-blur-sm overflow-hidden border-b border-[#2d2d2d]">
        {children}
      </div>
    );
    try {
      return (
        <TxContainer>
          <div className="mb-2 overflow-hidden">
            <TransactionLink
              status={"Completed"}
              transactionHash={JSON.parse(toolOutput.result)}
              error={null}
            />
          </div>
        </TxContainer>
      );
    } catch (e) {
      console.error("Failed to parse swap response:", e);
      return (
        <TxContainer>
          <div className="mb-2 overflow-hidden">
            <TransactionLink
              status={"Failed"}
              transactionHash={null}
              error={toolOutput.result}
            />
          </div>
        </TxContainer>
      );
    }
  }

  if (toolOutput.name === "get_quote") {
    try {
      // Clean up the result string in case it has extra escaping
      let resultData = toolOutput.result;

      // Sometimes quotes get double-stringified, check for that
      if (resultData.startsWith('"') && resultData.endsWith('"')) {
        try {
          // Try to parse once to remove outer quotes if double-stringified
          resultData = JSON.parse(resultData);
        } catch (e) {
          // If this fails, keep the original string
          resultData = toolOutput.result;
        }
      }

      try {
        // Parse the data to an object
        const parsedData =
          typeof resultData === "string" ? JSON.parse(resultData) : resultData;

        // First try Jupiter quote schema
        const jupiterQuote = JupiterQuoteResponseSchema.safeParse(parsedData);
        if (jupiterQuote.success) {
          return <JupiterQuoteDisplay quote={jupiterQuote.data} />;
        }

        // Then try regular quote schema
        const quote = QuoteResponseSchema.safeParse(parsedData);
        if (quote.success) {
          return <QuoteDisplay quote={quote.data} />;
        }

        // If neither schema matches, throw an error
        throw new Error("Failed to validate quote with either schema");
      } catch (parseError) {
        console.error("JSON parse error:", parseError);
        throw parseError;
      }
    } catch (e) {
      console.error("Quote processing failed:", e);

      return null;
      // return (
      //   <div className="bg-blue-900/20 text-blue-300 rounded-lg px-4 py-3 my-2 backdrop-blur-sm border border-opacity-20 border-blue-500">
      //     <p className="text-red-400 break-words">
      //       Failed to parse quote data:{" "}
      //       {e instanceof Error ? e.message : "Unknown error"}
      //     </p>
      //     <details>
      //       <summary className="cursor-pointer text-sm">
      //         View raw quote data
      //       </summary>
      //       <pre className="text-xs mt-2 overflow-x-auto p-2 bg-gray-800 rounded break-words whitespace-pre-wrap">
      //         {typeof toolOutput.result === "string"
      //           ? toolOutput.result
      //           : JSON.stringify(toolOutput.result, null, 2)}
      //       </pre>
      //     </details>
      //   </div>
      // );
    }
  }

  return null; // <ChatMessage message={toolOutput.result} direction="incoming" />;
};

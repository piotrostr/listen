import { Link } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { ChatMessage } from "../components/Messages";
import { Message } from "../hooks/useChat";

// TODO: Replace with actual chat history from a proper data store
const MOCK_CHAT_HISTORY: Message[] = [
  {
    id: "1",
    message: "How is my portfolio performing?",
    direction: "outgoing",
    timestamp: new Date(Date.now() - 1000 * 60 * 30), // 30 mins ago
    isToolCall: false,
  },
  {
    id: "1-response",
    message: "Your portfolio has shown a 5% increase over the last 24 hours...",
    direction: "incoming",
    timestamp: new Date(Date.now() - 1000 * 60 * 29),
    isToolCall: false,
  },
  {
    id: "2",
    message: "What's the best time to buy SOL?",
    direction: "outgoing",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2), // 2 hours ago
    isToolCall: false,
  },
  {
    id: "2-response",
    message:
      "Market analysis suggests SOL is currently in a consolidation phase...",
    direction: "incoming",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2 + 60000),
    isToolCall: false,
  },
  {
    id: "3",
    message: "Show me my recent transactions",
    direction: "outgoing",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24), // 1 day ago
    isToolCall: false,
  },
  {
    id: "3-response",
    message:
      "Here are your recent transactions:\n\n- Swap 1.5 SOL for USDC\n- Deposit 100 USDC to Marinade\n- Stake 2 SOL",
    direction: "incoming",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 + 60000),
    isToolCall: false,
  },
];

export function ChatHistory() {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-white">Chat History</h1>
        <Link
          to="/chat"
          className="px-4 py-2 bg-purple-500/20 hover:bg-purple-500/30 text-purple-300 rounded-lg transition-colors"
        >
          New Chat
        </Link>
      </div>

      <div className="space-y-8">
        {MOCK_CHAT_HISTORY.reduce<Array<{ date: string; messages: Message[] }>>(
          (acc, message) => {
            const date = message.timestamp.toDateString();
            const existingGroup = acc.find((group) => group.date === date);

            if (existingGroup) {
              existingGroup.messages.push(message);
            } else {
              acc.push({ date, messages: [message] });
            }

            return acc;
          },
          []
        ).map((group) => (
          <div key={group.date} className="space-y-4">
            <h2 className="text-sm text-gray-400 font-medium">
              {group.date === new Date().toDateString()
                ? "Today"
                : group.date === new Date(Date.now() - 86400000).toDateString()
                  ? "Yesterday"
                  : group.date}
            </h2>
            <div className="space-y-6">
              {group.messages.map((message) => (
                <div key={message.id} className="space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-gray-500">
                      {formatDistanceToNow(message.timestamp, {
                        addSuffix: true,
                      })}
                    </span>
                  </div>
                  <ChatMessage
                    message={message.message}
                    direction={message.direction}
                  />
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

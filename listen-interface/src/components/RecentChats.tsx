import { Link } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { Message } from "../hooks/useChat";

// Mock data - later this should be moved to a proper data store or context
const MOCK_CHAT_HISTORY: Message[] = [
  {
    id: "1",
    message: "How is my portfolio performing?",
    direction: "outgoing",
    timestamp: new Date(Date.now() - 1000 * 60 * 30), // 30 mins ago
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
    id: "3",
    message: "Show me my recent transactions",
    direction: "outgoing",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24), // 1 day ago
    isToolCall: false,
  },
];

interface RecentChatsProps {
  isSidebarOpen: boolean;
}

export function RecentChats({ isSidebarOpen }: RecentChatsProps) {
  if (!isSidebarOpen) return null;

  return (
    <div className="px-4 py-2">
      <div className="flex justify-between items-center mb-2">
        <h3 className="text-sm font-medium text-gray-300">Recent Chats</h3>
        <Link
          to="/chat-history"
          className="text-xs text-purple-400 hover:text-purple-300"
        >
          View all
        </Link>
      </div>
      <div className="space-y-2">
        {MOCK_CHAT_HISTORY.slice(0, 3).map((chat) => (
          <Link
            key={chat.id}
            to="/chat"
            className="block p-2 rounded-lg hover:bg-purple-500/10 transition-colors"
          >
            <p className="text-sm text-gray-300 truncate">{chat.message}</p>
            <p className="text-xs text-gray-500 mt-1">
              {formatDistanceToNow(chat.timestamp, { addSuffix: true })}
            </p>
          </Link>
        ))}
      </div>
    </div>
  );
}

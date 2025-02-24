import { Link } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { useEffect, useState } from "react";
import { Chat, chatCache } from "../hooks/cache";

export function ChatHistory() {
  const [chats, setChats] = useState<Chat[]>([]);

  useEffect(() => {
    // Load all chats from cache
    const loadChats = async () => {
      // This is a temporary solution - we should implement a method to get all chats
      const allChats = await chatCache.getAll();
      setChats(
        allChats.sort(
          (a, b) => b.lastMessageAt.getTime() - a.lastMessageAt.getTime()
        )
      );
    };

    loadChats();
  }, []);

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

      <div className="space-y-4">
        {chats.map((chat) => {
          const firstMessage = chat.messages[0];

          return (
            <Link
              key={chat.id}
              to="/chat"
              search={{ chatId: chat.id }}
              className="block p-4 bg-blue-900/20 rounded-lg hover:bg-blue-900/30 transition-colors"
            >
              <div className="flex justify-between items-start mb-2">
                <h3 className="text-lg text-blue-300 font-medium">
                  {chat.title || firstMessage?.message.slice(0, 50) + "..."}
                </h3>
                <span className="text-sm text-gray-400">
                  {formatDistanceToNow(chat.lastMessageAt, { addSuffix: true })}
                </span>
              </div>
              <p className="text-sm text-gray-300">
                {chat.messages.length} messages
              </p>
            </Link>
          );
        })}
      </div>
    </div>
  );
}

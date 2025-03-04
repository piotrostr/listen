import { Link } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { useEffect, useState } from "react";
import { chatCache } from "../hooks/localStorage";
import { Chat } from "../types/message";

export function RecentChats() {
  const [recentChats, setRecentChats] = useState<Chat[]>([]);

  useEffect(() => {
    const loadRecentChats = async () => {
      const allChats = await chatCache.getAll();
      if (allChats.length > 0) {
        const recent = allChats
          .sort(
            (a, b) =>
              (b.lastMessageAt.getTime() ?? 0) -
              (a.lastMessageAt.getTime() ?? 0)
          )
          .slice(0, 5); // Show up to 5 recent chats
        setRecentChats(recent);
      }
    };

    loadRecentChats();
  }, []);

  return (
    <div className="overflow-hidden transition-all duration-300 ease-in-out">
      {recentChats.map((chat) => (
        <Link
          key={chat.id}
          to="/chat"
          search={{ chatId: chat.id }}
          className="flex items-center h-10 px-4 text-sm text-gray-300 hover:text-white hover:bg-purple-500/10 transition-colors"
        >
          <div className="flex-1 min-w-0">
            <div className="truncate text-xs">
              {chat.title || chat.messages[0]?.message.slice(0, 20) + "..."}
            </div>
            <div className="text-[10px] text-gray-500">
              {formatDistanceToNow(chat.lastMessageAt, { addSuffix: true })}
            </div>
          </div>
        </Link>
      ))}
      {recentChats.length > 0 && (
        <Link
          to="/chat-history"
          className="flex items-center h-10 px-4 text-sm text-purple-400 hover:text-purple-300 hover:bg-purple-500/10 transition-colors"
        >
          View all chats
        </Link>
      )}
    </div>
  );
}

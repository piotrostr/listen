import { Link } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { useEffect, useState } from "react";
import { chatCache } from "../hooks/localStorage";
import { Chat } from "../hooks/types";

export function RecentChats({ isSidebarOpen }: { isSidebarOpen: boolean }) {
  const [recentChats, setRecentChats] = useState<Chat[]>([]);
  console.log(recentChats);

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
          .slice(0, 3);
        setRecentChats(recent);
      }
    };

    loadRecentChats();
  }, []);

  if (!isSidebarOpen) return null;

  return (
    <div className="space-y-1">
      <div className="flex justify-between items-center mb-2">
        <h3 className="text-sm font-medium text-gray-300">Recent Chats</h3>
        <Link
          to="/chat-history"
          className="text-xs text-purple-400 hover:text-purple-300"
        >
          View all
        </Link>
      </div>

      {recentChats.map((chat) => (
        <Link
          key={chat.id}
          to="/chat"
          search={{ chatId: chat.id }}
          className="flex items-center h-10 px-4 rounded-lg text-gray-300 hover:text-white hover:bg-purple-500/10 transition-colors"
        >
          <span className="truncate">
            {chat.title || chat.messages[0]?.message.slice(0, 30) + "..."}
          </span>
          <span className="ml-auto text-xs text-gray-500">
            {formatDistanceToNow(chat.lastMessageAt, { addSuffix: true })}
          </span>
        </Link>
      ))}
    </div>
  );
}

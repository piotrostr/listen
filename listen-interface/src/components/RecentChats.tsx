import { Link } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { useEffect, useState } from "react";
import { Chat, chatCache } from "../hooks/cache";

export function RecentChats({ isSidebarOpen }: { isSidebarOpen: boolean }) {
  const [recentChats, setRecentChats] = useState<Chat[]>([]);

  useEffect(() => {
    const loadRecentChats = async () => {
      const allChats = await chatCache.getAll();
      const recent = allChats
        .sort((a, b) => b.lastMessageAt.getTime() - a.lastMessageAt.getTime())
        .slice(0, 5);
      setRecentChats(recent);
    };

    loadRecentChats();
  }, []);

  if (!isSidebarOpen) return null;

  return (
    <div className="space-y-1">
      <h3 className="text-sm font-medium text-gray-400 px-4 mb-2">
        Recent Chats
      </h3>
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

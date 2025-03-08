import { useNavigate } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { useEffect, useState } from "react";
import { chatCache } from "../hooks/localStorage";
import i18n from "../i18n";
import { Chat } from "../types/message";

export function RecentChats({ onItemClick }: { onItemClick?: () => void }) {
  const [recentChats, setRecentChats] = useState<Chat[]>([]);
  const navigate = useNavigate();

  useEffect(() => {
    const loadRecentChats = async () => {
      const allChats = await chatCache.getAll();
      if (allChats.length > 0) {
        const recent = allChats.sort(
          (a, b) =>
            (b.lastMessageAt.getTime() ?? 0) - (a.lastMessageAt.getTime() ?? 0)
        );
        setRecentChats(recent);
      }
    };

    loadRecentChats();
  }, []);

  const getLocale = () => {
    return i18n.language.startsWith("zh") ? zhCN : undefined;
  };

  const selectChat = (chatId: string) => {
    navigate({ to: "/", search: { chatId }, replace: true });
    if (onItemClick) onItemClick();
  };

  return (
    <div className="overflow-y-auto max-h-[43vh] scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent transition-all duration-300 ease-in-out">
      {recentChats.map((chat) => (
        <div
          key={chat.id}
          onClick={() => selectChat(chat.id)}
          className="flex items-center h-10 px-4 text-sm text-gray-300 hover:text-white hover:bg-purple-500/10 transition-colors cursor-pointer"
        >
          <div className="flex-1 min-w-0">
            <div className="truncate text-xs">
              {chat.title || chat.messages[0]?.message.slice(0, 20) + "..."}
            </div>
            <div className="text-[10px] text-gray-500">
              {formatDistanceToNow(chat.lastMessageAt, {
                addSuffix: true,
                locale: getLocale(),
              })}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

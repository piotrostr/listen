import { Link, useNavigate } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { chatCache } from "../hooks/localStorage";
import i18n from "../i18n";
import { Chat } from "../types/message";

export function RecentChats() {
  const [recentChats, setRecentChats] = useState<Chat[]>([]);
  const navigate = useNavigate();

  const { t } = useTranslation();

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
          .slice(0, 10);
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
  };

  return (
    <div className="overflow-hidden transition-all duration-300 ease-in-out">
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
      {recentChats.length > 0 && (
        <Link
          to="/"
          className="flex items-center h-10 px-4 text-sm text-purple-400 hover:text-purple-300 hover:bg-purple-500/10 transition-colors"
        >
          {t("recent_chats.view_all_chats")}
        </Link>
      )}
    </div>
  );
}

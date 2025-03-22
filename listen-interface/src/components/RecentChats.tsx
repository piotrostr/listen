import { useNavigate } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { useEffect, useState } from "react";
import { useMobile } from "../contexts/MobileContext";
import { chatCache } from "../hooks/localStorage";
import i18n from "../i18n";
import { Chat } from "../types/message";

const DropdownMenu = ({
  onShare,
  onRename,
  onDelete,
}: {
  onShare: () => void;
  onRename: () => void;
  onDelete: () => void;
}) => {
  return (
    <div>
      <button onClick={onShare}>Share</button>
      <button onClick={onRename}>Rename</button>
      <button onClick={onDelete}>Delete</button>
    </div>
  );
};
export function RecentChats({ onItemClick }: { onItemClick?: () => void }) {
  const [recentChats, setRecentChats] = useState<Chat[]>([]);
  const navigate = useNavigate();
  const { isMobile, isVerySmallScreen } = useMobile();

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

  const renameChat = async (chatId: string, newName: string) => {
    const chat = await chatCache.get(chatId);
    if (chat) {
      chat.title = newName;
      await chatCache.set(chatId, chat);
      const index = recentChats.findIndex((c) => c.id === chatId);
      if (index !== -1) {
        const newChats = [...recentChats];
        newChats[index] = chat;
        setRecentChats(newChats);
      }
    }
  };

  const deleteChat = async (chatId: string) => {
    await chatCache.delete(chatId);
    const newChats = recentChats.filter((c) => c.id !== chatId);
    setRecentChats(newChats);
  };

  useEffect(() => {
    loadRecentChats();

    const handleChatUpdate = () => {
      loadRecentChats();
    };

    window.addEventListener("chatUpdated", handleChatUpdate);

    return () => {
      window.removeEventListener("chatUpdated", handleChatUpdate);
    };
  }, []);

  const getLocale = () => {
    return i18n.language.startsWith("zh") ? zhCN : undefined;
  };

  const selectChat = (chatId: string) => {
    navigate({ to: "/", search: { chatId }, replace: true });
    if (onItemClick) onItemClick();
  };

  return (
    <div
      className={`overflow-y-auto ${
        isMobile
          ? isVerySmallScreen
            ? "max-h-[16.5vh]"
            : "max-h-[28vh]"
          : "max-h-[43vh]"
      } scrollbar-thin scrollbar-thumb-[#212121] scrollbar-track-transparent transition-all duration-300 ease-in-out`}
    >
      {recentChats.map((chat) => (
        <div
          key={chat.id}
          onClick={() => selectChat(chat.id)}
          className="flex items-center h-10 px-4 text-sm text-gray-300 hover:text-white hover:bg-[#212121] transition-colors cursor-pointer"
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

import { useNavigate } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { useEffect, useRef, useState } from "react";
import { BsThreeDots } from "react-icons/bs";
import { useMobile } from "../contexts/MobileContext";
import { chatCache } from "../hooks/localStorage";
import i18n from "../i18n";
import { Chat } from "../types/message";

const DropdownMenu = ({
  onShare,
  onRename,
  onDelete,
}: {
  onShare: (e: React.MouseEvent) => void;
  onRename: (e: React.MouseEvent) => void;
  onDelete: (e: React.MouseEvent) => void;
}) => {
  return (
    <div className="absolute right-2 top-10 bg-[#1a1a1a] shadow-lg rounded py-1 z-10 min-w-[120px]">
      <button
        onClick={onShare}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors"
      >
        Share
      </button>
      <button
        onClick={onRename}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors"
      >
        Rename
      </button>
      <button
        onClick={onDelete}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors text-red-400"
      >
        Delete
      </button>
    </div>
  );
};

export function RecentChats({ onItemClick }: { onItemClick?: () => void }) {
  const [recentChats, setRecentChats] = useState<Chat[]>([]);
  const [openDropdownId, setOpenDropdownId] = useState<string | null>(null);
  const navigate = useNavigate();
  const { isMobile, isVerySmallScreen } = useMobile();
  const dropdownRef = useRef<HTMLDivElement>(null);

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

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setOpenDropdownId(null);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  const getLocale = () => {
    return i18n.language.startsWith("zh") ? zhCN : undefined;
  };

  const selectChat = (chatId: string) => {
    navigate({ to: "/", search: { chatId }, replace: true });
    if (onItemClick) onItemClick();
  };

  const handleShare = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    console.log(`Share chat ${chatId}`);
    setOpenDropdownId(null);
  };

  const handleRename = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const newName = prompt("Enter new name for this chat:");
    if (newName) {
      renameChat(chatId, newName);
    }
    setOpenDropdownId(null);
  };

  const handleDelete = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm("Are you sure you want to delete this chat?")) {
      deleteChat(chatId);
    }
    setOpenDropdownId(null);
  };

  const toggleDropdown = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setOpenDropdownId(openDropdownId === chatId ? null : chatId);
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
          className="relative flex items-center h-10 px-4 text-sm text-gray-300 hover:text-white hover:bg-[#212121] transition-colors cursor-pointer group"
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

          <div
            className="relative opacity-0 group-hover:opacity-100 transition-opacity"
            ref={openDropdownId === chat.id ? dropdownRef : null}
          >
            <button
              onClick={(e) => toggleDropdown(chat.id, e)}
              className="p-1 rounded-full hover:bg-[#333333] transition-colors shadow-sm"
            >
              <BsThreeDots className="text-gray-400 hover:text-white" />
            </button>

            {openDropdownId === chat.id && (
              <DropdownMenu
                onShare={(e) => handleShare(chat.id, e)}
                onRename={(e) => handleRename(chat.id, e)}
                onDelete={(e) => handleDelete(chat.id, e)}
              />
            )}
          </div>
        </div>
      ))}
    </div>
  );
}

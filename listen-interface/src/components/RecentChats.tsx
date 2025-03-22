import { useNavigate } from "@tanstack/react-router";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { BsThreeDots } from "react-icons/bs";
import { useMobile } from "../contexts/MobileContext";
import { useSidebar } from "../contexts/SidebarContext";
import { chatCache } from "../hooks/localStorage";
import i18n from "../i18n";
import { Chat } from "../types/message";

const DropdownMenu = ({
  onShare,
  onRename,
  onDelete,
  position,
  dropdownRef,
}: {
  onShare: (e: React.MouseEvent) => void;
  onRename: (e: React.MouseEvent) => void;
  onDelete: (e: React.MouseEvent) => void;
  position: { x: number; y: number } | null;
  dropdownRef: React.RefObject<HTMLDivElement>;
}) => {
  if (!position) return null;

  return createPortal(
    <div
      className="fixed bg-[#1a1a1a] shadow-lg rounded py-1 min-w-[120px] z-[1000]"
      style={{
        left: `${position.x}px`,
        top: `${position.y}px`,
      }}
      ref={dropdownRef}
    >
      <button
        onClick={onShare}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors rounded-lg"
      >
        Share
      </button>
      <button
        onClick={onRename}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors rounded-lg"
      >
        Rename
      </button>
      <button
        onClick={onDelete}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors text-red-400 rounded-lg"
      >
        Delete
      </button>
    </div>,
    document.body
  );
};

export function RecentChats({ onItemClick }: { onItemClick?: () => void }) {
  const { isSidebarOpen, setIsDropdownOpen } = useSidebar();
  const [recentChats, setRecentChats] = useState<Chat[]>([]);
  const [openDropdownId, setOpenDropdownId] = useState<string | null>(null);
  const [dropdownPosition, setDropdownPosition] = useState<{
    x: number;
    y: number;
  } | null>(null);
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
    setIsDropdownOpen(openDropdownId !== null);

    if (openDropdownId !== null) {
      const handleClickOutside = (event: MouseEvent) => {
        if (
          dropdownRef.current &&
          !dropdownRef.current.contains(event.target as Node)
        ) {
          closeDropdown();
        }
      };

      document.addEventListener("mousedown", handleClickOutside);
      return () => {
        document.removeEventListener("mousedown", handleClickOutside);
      };
    }
  }, [openDropdownId, setIsDropdownOpen]);

  useEffect(() => {
    return () => {
      setIsDropdownOpen(false);
    };
  }, [setIsDropdownOpen]);

  const getLocale = () => {
    return i18n.language.startsWith("zh") ? zhCN : undefined;
  };

  const selectChat = (chatId: string) => {
    navigate({ to: "/", search: { chatId }, replace: true });
    if (onItemClick) onItemClick();
  };

  const toggleDropdown = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (openDropdownId === chatId) {
      setOpenDropdownId(null);
      setDropdownPosition(null);
    } else {
      setOpenDropdownId(chatId);
      const button = e.currentTarget as HTMLElement;
      const rect = button.getBoundingClientRect();
      setDropdownPosition({
        x: rect.left - 100,
        y: rect.bottom + 5,
      });
    }
  };

  const closeDropdown = () => {
    setOpenDropdownId(null);
    setDropdownPosition(null);
  };

  const handleShare = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    console.log(`Share chat ${chatId}`);
    closeDropdown();
  };

  const handleRename = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const newName = prompt("Enter new name for this chat:");
    if (newName) {
      renameChat(chatId, newName);
    }
    closeDropdown();
  };

  const handleDelete = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm("Are you sure you want to delete this chat?")) {
      deleteChat(chatId);
    }
    closeDropdown();
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

          <div className="relative opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onClick={(e) => toggleDropdown(chat.id, e)}
              className="p-1 rounded-full hover:bg-[#333333] transition-colors shadow-sm"
            >
              <BsThreeDots className="text-gray-400 hover:text-white" />
            </button>
          </div>
        </div>
      ))}

      {openDropdownId && (
        <DropdownMenu
          onShare={(e) => handleShare(openDropdownId, e)}
          onRename={(e) => handleRename(openDropdownId, e)}
          onDelete={(e) => handleDelete(openDropdownId, e)}
          position={dropdownPosition}
          dropdownRef={dropdownRef}
        />
      )}
    </div>
  );
}

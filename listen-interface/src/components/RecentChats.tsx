import { useNavigate } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { BsThreeDots } from "react-icons/bs";
import { RiDeleteBin5Line, RiEdit2Line, RiShare2Line } from "react-icons/ri";
import { useChat } from "../contexts/ChatContext";
import { useModal } from "../contexts/ModalContext";
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

  const { t } = useTranslation();

  return createPortal(
    <div
      className="fixed bg-[#1a1a1a] shadow-lg rounded py-1 z-[1000]"
      style={{
        left: `${position.x}px`,
        top: `${position.y}px`,
      }}
      ref={dropdownRef}
    >
      <button
        onClick={onShare}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors rounded-lg flex items-center"
      >
        <RiShare2Line className="mr-2" />
        {t("share_modal.share")}
      </button>
      <button
        onClick={onRename}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors rounded-lg flex items-center"
      >
        <RiEdit2Line className="mr-2" />
        {t("share_modal.rename")}
      </button>
      <button
        onClick={onDelete}
        className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#2a2a2a] transition-colors text-red-400 rounded-lg flex items-center"
      >
        <RiDeleteBin5Line className="mr-2" />
        {t("share_modal.delete")}
      </button>
    </div>,
    document.body
  );
};

export function RecentChats({ onItemClick }: { onItemClick?: () => void }) {
  const { setIsDropdownOpen } = useSidebar();
  const { openShareModal } = useModal();
  const { shareChat } = useChat();
  const [recentChats, setRecentChats] = useState<Chat[]>([]);
  const [openDropdownId, setOpenDropdownId] = useState<string | null>(null);
  const [dropdownPosition, setDropdownPosition] = useState<{
    x: number;
    y: number;
  } | null>(null);
  const [editingChatId, setEditingChatId] = useState<string | null>(null);
  const [editingText, setEditingText] = useState("");
  const navigate = useNavigate();
  const dropdownRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const { t } = useTranslation();

  // Group chats by time periods
  const groupChatsByTimePeriod = (chats: Chat[]) => {
    const today = new Date();
    today.setHours(0, 0, 0, 0);

    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    const last7Days = new Date(today);
    last7Days.setDate(last7Days.getDate() - 7);

    const groups: { [key: string]: Chat[] } = {
      today: [],
      yesterday: [],
      last7Days: [],
    };

    // Object to store chats by month
    const monthGroups: { [key: string]: Chat[] } = {};

    chats.forEach((chat) => {
      const chatDate = new Date(chat.lastMessageAt);
      chatDate.setHours(0, 0, 0, 0);

      if (chatDate.getTime() === today.getTime()) {
        groups.today.push(chat);
      } else if (chatDate.getTime() === yesterday.getTime()) {
        groups.yesterday.push(chat);
      } else if (chatDate.getTime() > last7Days.getTime()) {
        groups.last7Days.push(chat);
      } else {
        // Group by month and year
        const monthYear = chatDate.toLocaleString(i18n.language, {
          month: "long",
          year: "numeric",
        });
        if (!monthGroups[monthYear]) {
          monthGroups[monthYear] = [];
        }
        monthGroups[monthYear].push(chat);
      }
    });

    return { timePeriods: groups, monthGroups };
  };

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

  const handleShare = async (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const sharedChatId = await shareChat(chatId, true); // cached: true
    const url = `${window.location.origin}/?chatId=${sharedChatId}&shared=true`;
    openShareModal(url);
    closeDropdown();
  };

  const handleRename = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const chat = recentChats.find((c) => c.id === chatId);
    if (chat) {
      setEditingChatId(chatId);
      setEditingText(
        chat.title || chat.messages[0]?.message.slice(0, 20) + "..." || ""
      );

      // Focus the textarea in the next tick after rendering
      setTimeout(() => {
        if (textareaRef.current) {
          textareaRef.current.focus();
          textareaRef.current.select();
        }
      }, 0);
    }
    closeDropdown();
  };

  const handleDelete = (chatId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    deleteChat(chatId);
    closeDropdown();
  };

  const saveRename = async () => {
    if (editingChatId && editingText.trim()) {
      await renameChat(editingChatId, editingText);
      setEditingChatId(null);
    }
  };

  const cancelRename = () => {
    setEditingChatId(null);
  };

  // Group the chats
  const { timePeriods, monthGroups } = groupChatsByTimePeriod(recentChats);

  const PeriodHeader = ({
    timePeriod,
    isMonthPeriod,
  }: {
    timePeriod: string;
    isMonthPeriod?: boolean;
  }) => {
    return (
      <div className="px-2 py-1 text-xs text-white font-semibold">
        {!isMonthPeriod ? t(`recent_chats.${timePeriod}`) : timePeriod}
      </div>
    );
  };

  return (
    <div className="overflow-y-auto h-full scrollbar-thin scrollbar-thumb-[#212121] scrollbar-track-transparent transition-all duration-300 ease-in-out">
      {/* Today's chats */}
      {timePeriods.today.length > 0 && (
        <>
          <PeriodHeader timePeriod="today" />
          {timePeriods.today.map((chat) => renderChatItem(chat))}
        </>
      )}

      {/* Yesterday's chats */}
      {timePeriods.yesterday.length > 0 && (
        <div className="mt-4">
          <PeriodHeader timePeriod="yesterday" />
          {timePeriods.yesterday.map((chat) => renderChatItem(chat))}
        </div>
      )}

      {/* Last 7 days chats */}
      {timePeriods.last7Days.length > 0 && (
        <div className="mt-4">
          <PeriodHeader timePeriod="last7Days" />
          {timePeriods.last7Days.map((chat) => renderChatItem(chat))}
        </div>
      )}

      {/* Older chats grouped by month */}
      {Object.entries(monthGroups).map(([monthYear, chats]) => (
        <div key={monthYear} className="mt-4">
          <PeriodHeader timePeriod={monthYear} isMonthPeriod={true} />
          {chats.map((chat) => renderChatItem(chat))}
        </div>
      ))}

      {recentChats.length === 0 && (
        <div className="py-2 text-xs text-gray-400">{t("no_recent_chats")}</div>
      )}

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

  // Helper function to render chat items
  function renderChatItem(chat: Chat) {
    return (
      <div
        key={chat.id}
        onClick={() => selectChat(chat.id)}
        className="relative flex items-center h-8 px-2 text-sm text-gray-300 hover:text-white hover:bg-[#212121] transition-colors cursor-pointer group rounded-lg"
      >
        <div className="flex-1 min-w-0">
          {editingChatId === chat.id ? (
            <textarea
              ref={textareaRef}
              className="w-full p-1 bg-[#2a2a2a] border border-[#3a3a3a] rounded text-xs text-white resize-none focus:outline-none focus:border-blue-500"
              rows={1}
              value={editingText}
              onClick={(e) => e.stopPropagation()}
              onChange={(e) => setEditingText(e.target.value)}
              onBlur={saveRename}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  saveRename();
                } else if (e.key === "Escape") {
                  cancelRename();
                }
              }}
            />
          ) : (
            <>
              <div className="truncate text-xs">
                {chat.title || chat.messages[0]?.message.slice(0, 20) + "..."}
              </div>
            </>
          )}
        </div>

        {editingChatId !== chat.id && (
          <div className="relative opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onClick={(e) => toggleDropdown(chat.id, e)}
              className="p-1 rounded-full hover:bg-[#333333] transition-colors shadow-sm"
            >
              <BsThreeDots className="text-gray-400 hover:text-white" />
            </button>
          </div>
        )}
      </div>
    );
  }
}

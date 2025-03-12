import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { chatCache } from "../hooks/localStorage";
import { Chat } from "../types/message";

export function ChatHistory() {
  const { t } = useTranslation();
  const [chats, setChats] = useState<Chat[]>([]);

  useEffect(() => {
    const loadChats = async () => {
      const allChats = await chatCache.getAll();
      setChats(allChats);
    };

    loadChats();
  }, []);

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-white">Chat History</h1>
        <Link
          to="/"
          params={{
            new: true,
          }}
          className="px-4 py-2 bg-[#2D2D2D] hover:bg-[#2D2D2D] text-white rounded-lg transition-colors"
        >
          {t("chat_history.new_chat")}
        </Link>
      </div>
      {/*chats.length === 0 && (
        <div className="text-gray-400">{t("chat_history.no_chats_found")}</div>
      )*/}

      <div className="space-y-4">
        {chats.length > 0 &&
          chats.map((chat) => {
            const firstMessage = chat?.messages?.[0];

            return (
              <div key={chat.id} className="group relative">
                <Link
                  to="/"
                  search={{ chatId: chat.id }}
                  className="block p-4 bg-blue-900/20 rounded-lg hover:bg-blue-900/30 transition-colors"
                >
                  <div className="flex justify-between items-start mb-2">
                    <h3 className="text-lg text-blue-300 font-medium">
                      {chat.title || firstMessage?.message.slice(0, 50) + "..."}
                    </h3>
                    <span className="text-sm text-gray-400"></span>
                  </div>
                  <p className="text-sm text-gray-300">
                    {chat.messages.length} messages
                  </p>
                </Link>
                <button
                  onClick={async (e) => {
                    e.preventDefault();
                    await chatCache.delete(chat.id);
                    setChats(chats.filter((c) => c.id !== chat.id));
                  }}
                  className="absolute top-4 right-4 p-2 bg-red-500/20 rounded-lg lg:opacity-0 group-hover:opacity-100 transition-opacity hover:bg-red-500/30"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    className="text-red-300"
                  >
                    <path d="M3 6h18"></path>
                    <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
                    <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
                  </svg>
                </button>
              </div>
            );
          })}
      </div>
    </div>
  );
}

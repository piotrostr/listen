import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { chatCache } from "../hooks/localStorage";
import { Chat } from "../hooks/types";

export function ChatHistory() {
  const [chats, setChats] = useState<Chat[]>([]);

  useEffect(() => {
    const loadChats = async () => {
      const allChats = await chatCache.getAll();
      console.log("ChatHistory", allChats);
      setChats(allChats);
    };

    loadChats();
  }, []);

  console.log("ChatHistory", chats);

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-white">Chat History</h1>
        <Link
          to="/chat"
          className="px-4 py-2 bg-purple-500/20 hover:bg-purple-500/30 text-purple-300 rounded-lg transition-colors"
        >
          New Chat
        </Link>
      </div>
      {chats.length === 0 && (
        <div className="text-gray-400">No chats found</div>
      )}

      <div className="space-y-4">
        {chats.length > 0 &&
          chats.map((chat) => {
            const firstMessage = chat?.messages?.[0];

            return (
              <Link
                key={chat.id}
                to="/chat"
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
            );
          })}
      </div>
    </div>
  );
}

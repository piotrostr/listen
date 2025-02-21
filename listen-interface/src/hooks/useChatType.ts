import { useQuery, useQueryClient } from "@tanstack/react-query";

export type ChatType = "solana" | "omni";

const CHAT_TYPE_KEY = ["chatType"];

export const useChatType = () => {
  const queryClient = useQueryClient();

  const { data: chatType = null } = useQuery<ChatType>({
    queryKey: CHAT_TYPE_KEY,
    initialData: (localStorage.getItem("chatType") as ChatType) || "solana",
  });

  const setChatType = (newChatType: ChatType) => {
    queryClient.setQueryData(CHAT_TYPE_KEY, newChatType);
    localStorage.setItem("chatType", newChatType);
  };

  return {
    chatType: chatType!,
    setChatType,
  };
};

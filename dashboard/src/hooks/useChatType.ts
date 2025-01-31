import { useQuery, useQueryClient } from "@tanstack/react-query";

export type ChatType = "evm" | "solana" | "pump";

const CHAT_TYPE_KEY = ["chatType"];

export const useChatType = () => {
  const queryClient = useQueryClient();

  const { data: chatType = null } = useQuery<ChatType>({
    queryKey: CHAT_TYPE_KEY,
    initialData: "solana",
  });

  const setChatType = (newChatType: ChatType) => {
    queryClient.setQueryData(CHAT_TYPE_KEY, newChatType);
  };

  return {
    chatType: chatType!,
    setChatType,
  };
};

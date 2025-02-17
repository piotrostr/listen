import { useQuery, useQueryClient } from "@tanstack/react-query";

export type ChatType = "evm" | "solana" | "omni";

const CHAT_TYPE_KEY = ["chatType"];

export const useChatType = () => {
  const queryClient = useQueryClient();

  const { data: chatType = null } = useQuery<ChatType>({
    queryKey: CHAT_TYPE_KEY,
    initialData: "omni", // hard set to omni, does all evm/sol does
  });

  const setChatType = (newChatType: ChatType) => {
    queryClient.setQueryData(CHAT_TYPE_KEY, newChatType);
  };

  return {
    chatType: chatType!,
    setChatType,
  };
};

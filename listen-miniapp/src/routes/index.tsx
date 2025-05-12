import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { Chat } from "../components/Chat";
import { GettingStarted } from "../components/GettingStarted";
import { Spinner } from "../components/Spinner";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";

export const Route = createFileRoute("/")({
  component: Index,
  validateSearch: (search: Record<string, unknown>) => {
    return z
      .object({
        chatId: z.string().optional(),
        new: z.boolean().optional(),
        shared: z.boolean().optional(),
        message: z.string().optional(),
      })
      .parse(search);
  },
});

function Index() {
  const { isAuthenticated, isLoading } = useIsAuthenticated();
  const { shared, chatId } = Route.useSearch();

  // Allow access to shared chats without authentication
  const isSharedChat = shared === true && chatId !== undefined;
  const shouldShowChat = isAuthenticated || isSharedChat;

  return (
    <div className="flex-1 overflow-hidden">
      {isLoading ? (
        <div className="flex items-center justify-center h-full">
          <Spinner />
        </div>
      ) : shouldShowChat ? (
        <Chat />
      ) : (
        <GettingStarted />
      )}
    </div>
  );
}

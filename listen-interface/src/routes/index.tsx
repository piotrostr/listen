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
      })
      .parse(search);
  },
});

function Index() {
  const { isAuthenticated, isLoading } = useIsAuthenticated();

  return (
    <div className="flex-1 overflow-hidden">
      {isLoading ? (
        <div className="flex items-center justify-center h-full">
          <Spinner />
        </div>
      ) : isAuthenticated ? (
        <Chat />
      ) : (
        <GettingStarted />
      )}
    </div>
  );
}

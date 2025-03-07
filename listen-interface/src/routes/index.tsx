import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { Chat } from "../components/Chat";
import { GettingStarted } from "../components/GettingStarted";
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
  const { isAuthenticated } = useIsAuthenticated();

  return (
    <div className="flex-1 overflow-hidden">
      {isAuthenticated ? <Chat /> : <GettingStarted />}
    </div>
  );
}

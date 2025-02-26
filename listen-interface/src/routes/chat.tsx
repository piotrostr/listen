import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";
import { Chat } from "../components/Chat";

const ChatSearchParamsSchema = z.object({
  chatId: z.string().optional(),
  new: z.boolean().optional(),
});

export const Route = createFileRoute("/chat")({
  component: RouteComponent,
  validateSearch: (search: Record<string, unknown>) => {
    return ChatSearchParamsSchema.parse(search);
  },
});

function RouteComponent() {
  return <Chat />;
}

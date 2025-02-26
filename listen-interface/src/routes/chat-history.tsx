import { createFileRoute } from "@tanstack/react-router";
import { ChatHistory } from "../components/ChatHistory";

export const Route = createFileRoute("/chat-history")({
  component: ChatHistory,
});

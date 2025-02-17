import { createFileRoute } from "@tanstack/react-router";
import { Chat } from "../components/Chat";

export const Route = createFileRoute("/chat")({
  component: ChatPage,
});

function ChatPage() {
  return (
    <div className="flex flex-col lg:flex-row gap-4 max-w-7xl mx-auto px-4 h-[calc(100vh-5rem)]">
      <div className="flex-1 h-full">
        <Chat />
      </div>
    </div>
  );
}

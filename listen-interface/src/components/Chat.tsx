import { useChat } from "../hooks/useChat";
import { useKeyboardInput } from "../hooks/useKeyboardInput";
import { ChatContainer } from "./ChatContainer";
import { MessageRenderer } from "./MessageRenderer";

const IS_DISABLED = process.env.NODE_ENV === "production";

const LoadingIndicator = () => (
  <div className="bg-purple-900/20 text-purple-300 rounded px-4 py-2">...</div>
);

export function Chat() {
  const { messages, isLoading, sendMessage, setMessages } = useChat();

  const { inputMessage } = useKeyboardInput({
    onSubmit: sendMessage,
    onClear: () => setMessages([]),
    isDisabled: IS_DISABLED,
  });

  if (IS_DISABLED) {
    return (
      <ChatContainer inputMessage="">
        <div className="text-purple-300 px-4 py-2">disabled</div>
      </ChatContainer>
    );
  }

  return (
    <ChatContainer inputMessage={inputMessage}>
      {messages.map((message) => (
        <MessageRenderer key={message.id} message={message} />
      ))}
      {isLoading && <LoadingIndicator />}
    </ChatContainer>
  );
}

export function ChatContainer({
  children,
  inputMessage,
}: {
  children: React.ReactNode;
  inputMessage: string;
}) {
  return (
    <div className="flex flex-col gap-4 h-[70vh] w-full max-w-4xl mx-auto px-4 font-mono">
      <div className="flex-1 overflow-hidden">
        <div className="h-full border-2 border-purple-500/30 rounded-lg overflow-hidden bg-black/40 backdrop-blur-sm">
          <div className="h-full flex flex-col">
            <div className="flex-1 overflow-y-auto p-4 scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
              {children}
            </div>
          </div>
        </div>
      </div>
      <ChatInput inputMessage={inputMessage} />
    </div>
  );
}

export function ChatInput({ inputMessage }: { inputMessage: string }) {
  return (
    <div className="h-12 border-2 border-purple-500/30 rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3">
      <span className="text-white whitespace-pre">{inputMessage}</span>
      <span className="w-2 h-5 bg-white terminal-blink ml-[1px]" />
    </div>
  );
}

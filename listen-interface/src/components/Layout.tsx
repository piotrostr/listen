import { Background } from "./Background";
import { Chat } from "./Chat";
import { Header } from "./Header";

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />
      <div className="relative z-10 py-20">
        <div className="flex flex-row gap-4 max-w-7xl mx-auto px-4">
          <div className="flex-1">{children}</div>
          <div className="w-[35%] min-w-[400px]">
            <Chat />
          </div>
        </div>
      </div>
    </div>
  );
}

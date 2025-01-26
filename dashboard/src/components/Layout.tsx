import { Background } from "./Background";
import { Header } from "./Header";

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="relative min-h-screen text-white">
      <Background />
      <Header />
      <div className="relative z-10 py-20">{children}</div>
    </div>
  );
}

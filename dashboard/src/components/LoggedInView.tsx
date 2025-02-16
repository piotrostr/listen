import { Chat } from "./Chat";
import { Portfolio } from "./Portfolio";

export function LoggedInView() {
  return (
    <div className="flex flex-col lg:flex-row gap-4 max-w-7xl mx-auto px-4 h-[calc(100vh-5rem)]">
      <div className="flex-1 h-full">
        <Chat />
      </div>
      <div className="w-full lg:w-80 h-full">
        <Portfolio />
      </div>
    </div>
  );
}

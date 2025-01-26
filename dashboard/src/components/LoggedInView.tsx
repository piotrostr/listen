import { Chat } from "./Chat";
import { Portfolio } from "./Portfolio";

export function LoggedInView() {
  return (
    <div className="flex flex-col lg:flex-row gap-4 max-w-7xl mx-auto px-4">
      <div className="flex-1">
        <Chat />
      </div>
      <div className="w-full lg:w-80">
        <Portfolio />
      </div>
    </div>
  );
}

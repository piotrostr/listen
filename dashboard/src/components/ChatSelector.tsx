import ethereumIcon from "../assets/icons/ethereum.svg";
import pumpIcon from "../assets/icons/pump.png";
import { ChatType } from "../hooks/useChatType";

interface ChatOptionProps {
  id: ChatType;
  iconPath: string;
  isSelected: boolean;
  onClick: (id: ChatType) => void;
}

function ChatOption({ id, iconPath, isSelected, onClick }: ChatOptionProps) {
  return (
    <button
      onClick={() => onClick(id)}
      className={`p-4 border-2 ${
        isSelected
          ? "border-purple-500 bg-purple-500/20"
          : "border-purple-500/30"
      } rounded-lg bg-black/40 backdrop-blur-sm hover:bg-purple-500/10 transition-all`}
    >
      <div className="flex flex-col items-center gap-2">
        <img src={iconPath} alt={id ?? "nothing"} className="w-6 h-6" />
      </div>
    </button>
  );
}

const CHAT_OPTIONS = [
  {
    id: "evm" as const,
    iconPath: ethereumIcon,
  },
  {
    id: "solana" as const,
    iconPath:
      "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
  },
  {
    id: "pump" as const,
    iconPath: pumpIcon,
  },
] as const;

interface ChatSelectorProps {
  selectedChat: ChatType;
  onSelectChat: (chat: ChatType) => void;
}

export function ChatSelector({
  selectedChat,
  onSelectChat,
}: ChatSelectorProps) {
  return (
    <div className="grid grid-cols-3 gap-4 mb-4 p-2">
      {CHAT_OPTIONS.map((option) => (
        <ChatOption
          key={option.id}
          id={option.id}
          iconPath={option.iconPath}
          isSelected={selectedChat === option.id}
          onClick={onSelectChat}
        />
      ))}
    </div>
  );
}

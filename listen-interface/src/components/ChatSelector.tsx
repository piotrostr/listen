import pumpIcon from "../assets/icons/pump.png";
import { ChatType } from "../hooks/useChatType";

interface ChatOptionProps {
  id: ChatType;
  iconPaths: readonly string[];
  isSelected: boolean;
  onClick: (id: ChatType) => void;
}

function ChatOption({ id, iconPaths, isSelected, onClick }: ChatOptionProps) {
  return (
    <button
      onClick={() => onClick(id)}
      disabled={id === "omni" && process.env.NODE_ENV === "production"}
      className={`p-4 border-2 ${
        isSelected
          ? "border-purple-500 bg-purple-500/20"
          : "border-purple-500/30"
      } rounded-lg bg-black/40 backdrop-blur-sm hover:bg-purple-500/10 transition-all`}
    >
      <div className="flex flex-row justify-center gap-2">
        {iconPaths.map((iconPath) => (
          <img
            key={iconPath}
            src={iconPath}
            alt={id ?? "nothing"}
            className="w-6 h-6"
          />
        ))}
      </div>
    </button>
  );
}

const CHAT_OPTIONS = [
  {
    id: "solana" as const,
    iconPaths: [
      "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
      pumpIcon,
    ],
  },
  {
    id: "omni" as const,
    iconPaths: [
      "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
      "https://dd.dexscreener.com/ds-data/chains/base.png",
      "https://dd.dexscreener.com/ds-data/chains/ethereum.png",
      "https://dd.dexscreener.com/ds-data/chains/arbitrum.png",
      // "https://dd.dexscreener.com/ds-data/chains/bsc.png",
    ],
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
    <div className="flex flex-row gap-2">
      {CHAT_OPTIONS.map((option) => (
        <ChatOption
          key={option.id}
          id={option.id}
          iconPaths={option.iconPaths}
          isSelected={selectedChat === option.id}
          onClick={onSelectChat}
        />
      ))}
    </div>
  );
}

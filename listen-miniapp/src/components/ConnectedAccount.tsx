import { TbPlugConnected } from "react-icons/tb";

interface ConnectedAccountProps {
  icon: React.ReactNode;
  isConnected: boolean;
  onConnect: () => void;
  value: string;
}

export function ConnectedAccount({
  icon: Icon,
  isConnected,
  onConnect,
  value,
}: ConnectedAccountProps) {
  return (
    <div
      className={`
        flex items-center p-1.5 rounded-lg 
        ${isConnected ? "flex-1 min-w-[200px]" : "w-fit"} 
        transition-colors
      `}
    >
      <div className="flex items-center">
        <div className="p-1 rounded-lg">{Icon}</div>
      </div>
      {isConnected ? (
        <>
          <div className="ml-1 p-1 rounded-lg text-green-500">
            <svg
              className="w-3 h-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M5 13l4 4L19 7"
              />
            </svg>
          </div>
          {value && (
            <span className="ml-1 text-xs text-gray-400 truncate max-w-[100px]">
              {value}
            </span>
          )}
        </>
      ) : (
        <button
          onClick={onConnect}
          className="ml-1 p-1 rounded-lg bg-[#2D2D2D] hover:bg-[#2D2D2D] text-white transition-colors"
        >
          <TbPlugConnected className="w-3 h-3" />
        </button>
      )}
    </div>
  );
}

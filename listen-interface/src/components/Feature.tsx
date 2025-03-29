import { ReactNode } from "react";

interface FeatureProps {
  isEnabled: boolean;
  onToggle: () => void;
  icon: ReactNode;
  label: string;
  enabledColors: {
    bgLight: string;
    textLight: string;
    bgDark: string;
    textDark: string;
  };
}

export function Feature({
  isEnabled,
  onToggle,
  icon,
  label,
  enabledColors,
}: FeatureProps) {
  return (
    <div>
      <span className="inline-block">
        <div
          className={`inline-flex h-9 rounded-full text-[13px] bg-[#2d2d2d]/40 font-medium focus-visible:outline-black text-gray-400 ${
            isEnabled
              ? `bg-[${enabledColors.bgLight}] text-${enabledColors.textLight} dark:bg-[${enabledColors.bgDark}] dark:text-[${enabledColors.textDark}]`
              : ""
          }`}
        >
          <button
            className="flex h-full min-w-8 items-center justify-center p-2"
            aria-pressed={isEnabled}
            aria-label={label}
            onClick={onToggle}
          >
            {icon}
            <div className="whitespace-nowrap pl-1 pr-1 hidden sm:inline">
              {label}
            </div>
          </button>
        </div>
      </span>
    </div>
  );
}

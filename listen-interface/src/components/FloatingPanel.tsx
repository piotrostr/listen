import { ReactNode } from "react";
import { useTranslation } from "react-i18next";

interface FloatingPanelProps {
  title: string;
  children: ReactNode;
  onClose: () => void;
  headerContent?: ReactNode;
}

export function FloatingPanel({
  title,
  children,
  onClose,
  headerContent,
}: FloatingPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="absolute right-4 top-4 bottom-4 w-[420px] rounded-xl border border-purple-500/30 bg-black/40 backdrop-blur-sm shadow-lg">
      <div className="flex flex-col h-full">
        <div className="border-b border-purple-500/30 p-3">
          <div className="flex justify-between items-center mb-2">
            <div>{t(`layout.${title}`)}</div>
            <button
              onClick={onClose}
              className="p-1 rounded-full hover:bg-purple-500/20"
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
          {headerContent && <div>{headerContent}</div>}
        </div>

        <div className="flex-1 overflow-y-auto pt-2">{children}</div>
      </div>
    </div>
  );
}

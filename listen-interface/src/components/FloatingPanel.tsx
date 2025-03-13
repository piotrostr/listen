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
    <div className="h-[90vh] w-[440px] rounded-xl border border-[#2D2D2D] backdrop-blur-sm shadow-lg bg-black/80">
      <div className="flex flex-col h-full">
        <div className="border-b border-[#2D2D2D] h-12">
          <div className="flex items-center justify-between gap-2 px-3 h-full">
            <div>{t(`layout.${title}`)}</div>
            <div className="flex items-center gap-2 h-full">
              {headerContent}
              <button
                onClick={onClose}
                className="w-8 h-8 rounded-full hover:bg-[#2D2D2D] flex items-center justify-center"
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
          </div>
        </div>

        <div className="flex-1 overflow-y-auto pt-2">{children}</div>
      </div>
    </div>
  );
}

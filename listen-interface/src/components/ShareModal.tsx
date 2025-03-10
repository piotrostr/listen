import { useRef } from "react";
import { useTranslation } from "react-i18next";

interface ShareModalProps {
  url: string;
  onClose: () => void;
}

export function ShareModal({ url, onClose }: ShareModalProps) {
  const urlInputRef = useRef<HTMLInputElement>(null);

  const { t } = useTranslation();

  const handleCopyClick = () => {
    if (urlInputRef.current) {
      urlInputRef.current.select();
      navigator.clipboard.writeText(url);

      // Flash the input to show it was copied
      urlInputRef.current.classList.add("bg-purple-500/20");
      setTimeout(() => {
        if (urlInputRef.current) {
          urlInputRef.current.classList.remove("bg-purple-500/20");
        }
      }, 300);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-gray-900 border border-[#2D2D2D] rounded-lg max-w-md w-full p-6 shadow-xl">
        <h3 className="text-xl font-medium text-white mb-4">
          {t("share_modal.share_this_chat")}
        </h3>
        <p className="text-gray-300 mb-4">
          {t("share_modal.anyone_with_this_link_can_view_this_chat")}
        </p>

        <div className="flex gap-2 mb-6">
          <input
            ref={urlInputRef}
            type="text"
            value={url}
            readOnly
            className="flex-1 bg-black/40 border border-[#2D2D2D] rounded px-3 py-2 text-white transition-colors"
          />
          <button
            onClick={handleCopyClick}
            className="bg-purple-500/20 hover:bg-purple-500/40 text-purple-300 px-4 py-2 rounded transition-colors"
          >
            {t("share_modal.copy")}
          </button>
        </div>

        <div className="flex justify-between">
          <button
            onClick={onClose}
            className="bg-transparent hover:bg-gray-800 text-gray-300 px-4 py-2 rounded transition-colors"
          >
            {t("share_modal.close")}
          </button>

          <a
            href={url}
            target="_blank"
            rel="noopener noreferrer"
            className="bg-blue-500/20 hover:bg-blue-500/40 text-blue-300 px-4 py-2 rounded transition-colors"
          >
            {t("share_modal.open_in_new_tab")}
          </a>
        </div>
      </div>
    </div>
  );
}

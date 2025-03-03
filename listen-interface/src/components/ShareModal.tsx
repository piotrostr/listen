import { useRef } from "react";

interface ShareModalProps {
  url: string;
  onClose: () => void;
}

export function ShareModal({ url, onClose }: ShareModalProps) {
  const urlInputRef = useRef<HTMLInputElement>(null);

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
      <div className="bg-gray-900 border border-purple-500/30 rounded-lg max-w-md w-full p-6 shadow-xl">
        <h3 className="text-xl font-medium text-white mb-4">Share this chat</h3>
        <p className="text-gray-300 mb-4">
          Anyone with this link can view this chat:
        </p>

        <div className="flex gap-2 mb-6">
          <input
            ref={urlInputRef}
            type="text"
            value={url}
            readOnly
            className="flex-1 bg-black/40 border border-purple-500/30 rounded px-3 py-2 text-white transition-colors"
          />
          <button
            onClick={handleCopyClick}
            className="bg-purple-500/20 hover:bg-purple-500/40 text-purple-300 px-4 py-2 rounded transition-colors"
          >
            Copy
          </button>
        </div>

        <div className="flex justify-between">
          <button
            onClick={onClose}
            className="bg-transparent hover:bg-gray-800 text-gray-300 px-4 py-2 rounded transition-colors"
          >
            Close
          </button>

          <a
            href={url}
            target="_blank"
            rel="noopener noreferrer"
            className="bg-blue-500/20 hover:bg-blue-500/40 text-blue-300 px-4 py-2 rounded transition-colors"
          >
            Open in new tab
          </a>
        </div>
      </div>
    </div>
  );
}

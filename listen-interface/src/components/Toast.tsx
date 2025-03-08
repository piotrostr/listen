import { useEffect } from "react";

export type ToastType = "error" | "success" | "info";

interface ToastProps {
  message: string;
  type: ToastType;
  onClose: () => void;
  isVisible: boolean;
}

export function Toast({ message, type, onClose, isVisible }: ToastProps) {
  useEffect(() => {
    const timer = setTimeout(() => {
      onClose();
    }, 5000); // Auto dismiss after 5 seconds

    return () => clearTimeout(timer);
  }, [onClose]);

  const baseClasses = "fixed top-4 left-1/2 transform -translate-x-1/2 z-50";

  const typeClasses = {
    error: "bg-red-500/20 text-red-300 border border-red-500/30",
    success: "bg-green-500/20 text-green-300 border border-green-500/30",
    info: "bg-blue-500/20 text-blue-300 border border-blue-500/30",
  };

  return (
    <div
      className={`${baseClasses} ${typeClasses[type]} px-6 py-3 rounded-lg shadow-lg flex items-center gap-2 transition-all duration-300 ease-in-out ${
        isVisible ? "opacity-100" : "opacity-0"
      }`}
    >
      <span>{message}</span>
      <button
        onClick={onClose}
        className="ml-2 hover:text-white transition-colors"
      >
        ✕
      </button>
    </div>
  );
}

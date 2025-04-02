import { useEffect } from "react";
import {
  FaCheckCircle,
  FaExclamationCircle,
  FaInfoCircle,
} from "react-icons/fa";
import { IoCloseOutline } from "react-icons/io5";

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

  const getIcon = () => {
    switch (type) {
      case "error":
        return <FaExclamationCircle className="text-red-500 text-lg w-4 h-4" />;
      case "success":
        return <FaCheckCircle className="text-green-500 text-lg w-4 h-4" />;
      case "info":
        return <FaInfoCircle className="text-blue-500 text-lg w-4 h-4" />;
    }
  };

  return (
    <div
      className={`${baseClasses} bg-[#151518] border border-[#2d2d2d] p-3 rounded-lg flex 
      items-center gap-3 transition-all duration-300 ease-in-out ${
        isVisible ? "opacity-100" : "opacity-0"
      }`}
    >
      {getIcon()}
      <span className="text-sm">{message}</span>
      <button
        onClick={onClose}
        className="bg-black/40 text-white rounded-lg w-7 h-7 flex 
        items-center justify-center hover:bg-[#2d2d2d] ml-2"
      >
        <IoCloseOutline className="w-5 h-5" />
      </button>
    </div>
  );
}

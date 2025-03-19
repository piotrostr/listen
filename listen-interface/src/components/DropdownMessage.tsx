import { useState } from "react";
import { FaChevronDown, FaChevronUp } from "react-icons/fa";
import { ChatMessage } from "./ChatMessage";

interface DropdownMessageProps {
  title: string;
  message: string;
  icon?: React.ReactNode;
}

const DropdownMessage = ({ title, message, icon }: DropdownMessageProps) => {
  const [isExpanded, setIsExpanded] = useState(false);

  const toggleExpand = () => {
    setIsExpanded(!isExpanded);
  };

  return (
    <div className="rounded-lg px-2 py-1 my-2 backdrop-blur-sm border border-[#2D2D2D] text-sm">
      <div
        className="flex items-center justify-between p-2 cursor-pointer"
        onClick={toggleExpand}
      >
        <div className="flex items-center gap-2">
          {icon && <span>{icon}</span>}
          <span className="font-medium">{title}</span>
        </div>
        <span>
          {isExpanded ? <FaChevronUp size={14} /> : <FaChevronDown size={14} />}
        </span>
      </div>

      {isExpanded && (
        <div className="p-2 border-t border-[#2D2D2D]">
          <ChatMessage message={message} direction="agent" />
        </div>
      )}
    </div>
  );
};

export default DropdownMessage;

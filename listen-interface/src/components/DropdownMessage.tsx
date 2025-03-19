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
    <div className="border border-gray-700 rounded-md overflow-hidden mb-2">
      <div
        className="flex items-center justify-between p-3 bg-gray-800 cursor-pointer hover:bg-gray-700"
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
        <div className="p-3 bg-gray-900 border-t border-gray-700">
          <ChatMessage message={message} direction="agent" />
        </div>
      )}
    </div>
  );
};

export default DropdownMessage;

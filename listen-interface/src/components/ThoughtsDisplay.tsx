import { useState } from "react";
import { useTranslation } from "react-i18next";
import { FaChevronDown, FaChevronRight } from "react-icons/fa";
import { TfiThought } from "react-icons/tfi";
import { Markdown } from "./ChatMessage";

export function ThoughtsDisplay({ thought }: { thought: string }) {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(false);
  const [isHovering, setIsHovering] = useState(false);

  const toggleExpand = () => {
    setIsExpanded(!isExpanded);
  };

  return (
    <div className="text-sm pl-2 text-gray-400">
      <div
        className="flex items-center justify-between p-2 cursor-pointer"
        onClick={toggleExpand}
        onMouseEnter={() => setIsHovering(true)}
        onMouseLeave={() => setIsHovering(false)}
      >
        <div
          className={`flex items-center gap-2 ${isExpanded ? "text-white" : ""}`}
        >
          <span>
            {isExpanded ? (
              <FaChevronDown size={14} />
            ) : isHovering ? (
              <FaChevronRight size={14} />
            ) : (
              <TfiThought size={14} />
            )}
          </span>
          <span>{t("tool_calls.thoughts")}</span>
        </div>
      </div>

      {isExpanded && (
        <div className="p-2 border-t border-[#2D2D2D]">
          <Markdown message={thought} />
        </div>
      )}
    </div>
  );
}

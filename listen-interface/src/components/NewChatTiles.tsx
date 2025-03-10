import { motion } from "framer-motion";
import React from "react";

interface Question {
  question: string;
  enabled: boolean;
  display: string;
}

interface NewChatTilesProps {
  questions: Question[];
  onSelect: (question: string) => void;
}

export const NewChatTiles: React.FC<NewChatTilesProps> = ({
  questions,
  onSelect,
}) => {
  return (
    <div className="w-full overflow-x-auto scrollbar-hide px-4 md:px-0">
      <div className="flex flex-nowrap gap-3 pb-4 min-w-min">
        {questions.map((item, index) => (
          <motion.div
            key={index}
            className="flex-none snap-start"
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <div
              onClick={() => onSelect(item.question)}
              className="flex flex-row items-start p-5 w-[173px] h-[80px] bg-transparent
                       border border-[#2D2D2D] rounded-[20px] cursor-pointer"
            >
              <div className="flex flex-col justify-center">
                <span className="font-space-grotesk text-base text-white line-clamp-2">
                  {item.display}
                </span>
              </div>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
};

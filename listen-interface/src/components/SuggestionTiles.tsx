import { motion } from "framer-motion";

const sanitizeSuggestion = (suggestion: string) => {
  return suggestion
    .replace(/^"|\s*"$/g, "") // Remove quotes (including when right before a question mark)
    .replace(/^[-*•]\s+/g, "") // Remove bullet points and dashes
    .replace(/^\d+[.)]?\s+/g, ""); // Remove numbered items like 1. or 1)
};

export const SuggestionTiles = ({
  suggestions,
  handleQuestionClick,
}: {
  suggestions: string[];
  handleQuestionClick: (question: string) => void;
}) => {
  return (
    <div className="w-full overflow-x-auto scrollbar-hide px-4 md:px-0">
      <div className="flex flex-nowrap gap-3 min-w-min md:flex md:justify-center pt-2 bg-gradient-to-b from-[#151518]/10 via-[#151518]/40 to-[#151518]/80 backdrop-blur-sm rounded-lg">
        {suggestions.map((suggestion, index) => (
          <motion.div
            key={index}
            className="flex-none snap-start"
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <div
              onClick={() =>
                handleQuestionClick(sanitizeSuggestion(suggestion))
              }
              className="flex-row min-w-[130px] max-w-[280px] h-[45px]
                             border border-[#2D2D2D] rounded-[20px] cursor-pointer 
                             flex justify-center items-center p-2 bg-[#151518]"
            >
              <span className="font-space-grotesk text-xs text-white line-clamp-2 text-center">
                {sanitizeSuggestion(suggestion)}
              </span>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
};

import { motion } from "framer-motion";
import React, { useEffect, useRef, useState } from "react";

interface Question {
  question: string;
  enabled: boolean;
}

interface NewChatCarouselProps {
  questions: Question[];
  onSelect: (question: string) => void;
}

export const NewChatCarousel: React.FC<NewChatCarouselProps> = ({
  questions,
  onSelect,
}) => {
  const [activeIndex, setActiveIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const itemHeight = 60;

  const handleScroll = () => {
    if (containerRef.current) {
      const scrollPosition = containerRef.current.scrollTop;
      const totalHeight = questions.length * itemHeight;

      // If we scroll past the middle set, reset to the equivalent position in the middle
      if (scrollPosition >= totalHeight * 2) {
        containerRef.current.scrollTop = scrollPosition - totalHeight;
      } else if (scrollPosition < totalHeight) {
        containerRef.current.scrollTop = scrollPosition + totalHeight;
      }

      const newIndex = Math.round((scrollPosition % totalHeight) / itemHeight);
      setActiveIndex(newIndex);
    }
  };

  useEffect(() => {
    const container = containerRef.current;
    if (container) {
      // Start in the middle set of items
      container.scrollTop = questions.length * itemHeight;
      container.addEventListener("scroll", handleScroll);
      return () => container.removeEventListener("scroll", handleScroll);
    }
  }, [questions.length]);

  const handleClick = (index: number) => {
    setActiveIndex(index);
    if (containerRef.current) {
      const middleSetOffset = questions.length * itemHeight;
      containerRef.current.scrollTo({
        top: middleSetOffset + index * itemHeight,
        behavior: "smooth",
      });
    }
    onSelect(questions[index].question);
  };

  const getVisibilityClass = (index: number) => {
    const normalizedActiveIndex = activeIndex % questions.length;
    const distance = Math.min(
      Math.abs(index - normalizedActiveIndex),
      Math.abs(index - normalizedActiveIndex - questions.length),
      Math.abs(index - normalizedActiveIndex + questions.length)
    );

    if (distance === 0) return "text-white opacity-100";
    if (distance === 1) return "text-gray-400 opacity-70";
    if (distance === 2) return "text-gray-600 opacity-30";
    return "opacity-0";
  };

  // Create array with 3 sets of questions for smooth infinite scroll
  const repeatedQuestions = [...questions, ...questions, ...questions];

  return (
    <div className="flex flex-row items-center justify-center">
      <div className="relative h-[300px]">
        <div
          ref={containerRef}
          className="h-full overflow-y-auto scrollbar-hide"
          style={{
            scrollSnapType: "y mandatory",
          }}
        >
          <div className="py-[120px]">
            {repeatedQuestions.map((item, index) => (
              <motion.div
                key={index}
                className={`flex items-center justify-between h-[60px] cursor-pointer px-4
                ${getVisibilityClass(index % questions.length)}
                transition-all duration-200 ease-in-out`}
                style={{
                  scrollSnapAlign: "center",
                }}
                onClick={() => handleClick(index % questions.length)}
              >
                <span className="text-lg flex-1 text-center">
                  {item.question}
                </span>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
      <span className="text-white">→</span>
    </div>
  );
};

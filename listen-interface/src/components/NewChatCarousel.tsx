import { motion } from "framer-motion";
import React, { useEffect, useRef, useState } from "react";

declare global {
  interface HTMLDivElement {
    scrollTimeout: number;
  }
}

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

      // Reset to middle when reaching edges
      if (scrollPosition >= totalHeight * 2) {
        containerRef.current.scrollTop = scrollPosition - totalHeight;
      } else if (scrollPosition < totalHeight) {
        containerRef.current.scrollTop = scrollPosition + totalHeight;
      }

      // When scrolling stops, snap to nearest item without triggering selection
      clearTimeout(containerRef.current.scrollTimeout);
      containerRef.current.scrollTimeout = setTimeout(() => {
        const newIndex =
          Math.round(scrollPosition / itemHeight) % questions.length;
        setActiveIndex(newIndex);

        // Just scroll to position without triggering selection
        const middleSetOffset = questions.length * itemHeight;
        containerRef.current?.scrollTo({
          top: middleSetOffset + newIndex * itemHeight,
          behavior: "smooth",
        });
      }, 50) as any;

      const newIndex = Math.round((scrollPosition % totalHeight) / itemHeight);
      setActiveIndex(newIndex);
    }
  };

  useEffect(() => {
    const container = containerRef.current;
    if (container) {
      // Start in the middle set
      container.scrollTop = questions.length * itemHeight;
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
    const normalizedIndex = index % questions.length;
    const distance = Math.min(
      Math.abs(normalizedIndex - activeIndex),
      Math.abs(normalizedIndex - activeIndex - questions.length),
      Math.abs(normalizedIndex - activeIndex + questions.length)
    );

    if (distance === 0) return "text-white opacity-100 scale-100";
    if (distance === 1) return "text-gray-400 opacity-70 scale-95";
    if (distance === 2) return "text-gray-600 opacity-40 scale-90";
    return "opacity-0 scale-85";
  };

  // Triple the questions for infinite scroll effect
  const repeatedQuestions = [...questions, ...questions, ...questions];

  return (
    <div className="flex flex-row items-center justify-center">
      <div className="relative h-[300px] w-[400px]">
        <div
          ref={containerRef}
          className="h-full overflow-y-auto scrollbar-hide"
          onScroll={handleScroll}
          style={{
            scrollSnapType: "y mandatory",
          }}
        >
          <div className="py-[120px]">
            {repeatedQuestions.map((item, index) => (
              <motion.div
                key={index}
                className={`flex items-center justify-center h-[60px] cursor-pointer px-4 transition-all duration-200
                  ${getVisibilityClass(index)}`}
                style={{
                  scrollSnapAlign: "center",
                }}
                onClick={() => handleClick(index % questions.length)}
              >
                <span className="text-lg text-center">{item.question}</span>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
      <motion.span
        className="text-white text-2xl cursor-pointer ml-4"
        whileHover={{ scale: 1.2 }}
        onClick={() => handleClick((activeIndex + 1) % questions.length)}
      >
        →
      </motion.span>
    </div>
  );
};

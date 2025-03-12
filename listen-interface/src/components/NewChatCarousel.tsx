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
  const [isScrolling, setIsScrolling] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const scrollTimeoutRef = useRef<number | null>(null);
  const itemHeight = 60;

  // Initialize scroll position to middle set
  useEffect(() => {
    const container = containerRef.current;
    if (container) {
      container.scrollTop = questions.length * itemHeight;
    }
  }, [questions.length]);

  const handleScroll = () => {
    if (!containerRef.current) return;

    const container = containerRef.current;
    const scrollPosition = container.scrollTop;
    const totalHeight = questions.length * itemHeight;

    // During active scrolling, only update the active index
    const newIndex = Math.floor(
      (container.scrollTop % totalHeight) / itemHeight
    );
    if (newIndex >= 0 && newIndex < questions.length) {
      setActiveIndex(newIndex);
    }

    // Set scrolling state immediately for click prevention
    setIsScrolling(true);

    // Debounce scroll handling for fast scrolls
    if (scrollTimeoutRef.current) {
      window.clearTimeout(scrollTimeoutRef.current);
    }

    // Only handle boundary resets AFTER scrolling has stopped
    scrollTimeoutRef.current = window.setTimeout(() => {
      // Check if we're near boundaries
      if (scrollPosition >= totalHeight * 1.8) {
        // If near bottom boundary, reset to middle without animation
        container.style.scrollBehavior = "auto";
        container.scrollTop = totalHeight + (scrollPosition % totalHeight);
        container.style.scrollBehavior = "smooth";
      } else if (scrollPosition <= totalHeight * 0.2) {
        // If near top boundary, reset to middle without animation
        container.style.scrollBehavior = "auto";
        container.scrollTop = totalHeight + (scrollPosition % totalHeight);
        container.style.scrollBehavior = "smooth";
      }

      setIsScrolling(false);
    }, 150); // Longer timeout to ensure scrolling has fully stopped
  };

  const handleClick = (index: number) => {
    // Don't process clicks while scrolling
    if (isScrolling) return;

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
      <div className="relative h-[300px] w-full">
        <div
          ref={containerRef}
          className="h-full overflow-y-auto scrollbar-hide"
          onScroll={handleScroll}
          style={{
            scrollSnapType: "y mandatory",
            scrollBehavior: "smooth",
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

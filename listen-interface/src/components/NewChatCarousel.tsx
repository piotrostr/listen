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
  const itemHeight = 60; // Height of each question item

  const handleScroll = () => {
    if (containerRef.current) {
      const scrollPosition = containerRef.current.scrollTop;
      const newIndex = Math.round(scrollPosition / itemHeight);
      setActiveIndex(Math.min(Math.max(0, newIndex), questions.length - 1));
    }
  };

  useEffect(() => {
    const container = containerRef.current;
    if (container) {
      container.addEventListener("scroll", handleScroll);
      return () => container.removeEventListener("scroll", handleScroll);
    }
  }, []);

  const handleClick = (index: number) => {
    setActiveIndex(index);
    if (containerRef.current) {
      containerRef.current.scrollTo({
        top: index * itemHeight,
        behavior: "smooth",
      });
    }
    onSelect(questions[index].question);
  };

  return (
    <div
      ref={containerRef}
      className="h-[300px] overflow-y-auto scrollbar-hide"
      style={{
        scrollSnapType: "y mandatory",
      }}
    >
      <div className="py-[120px]">
        {" "}
        {/* Add padding to allow centering */}
        {questions.map((item, index) => (
          <motion.div
            key={index}
            className={`flex items-center justify-center h-[60px] cursor-pointer px-4
              ${index === activeIndex ? "text-white" : "text-gray-500"}
              transition-colors duration-200 ease-in-out`}
            style={{
              scrollSnapAlign: "center",
            }}
            onClick={() => handleClick(index)}
          >
            <div className="flex items-center gap-2 text-center">
              {index === activeIndex && <span>→</span>}
              <span className="text-lg">{item.question}</span>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
};

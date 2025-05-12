import React, { useRef, useState } from "react";

interface SwipeHandlerProps {
  onOpenSidebar: () => void;
  isOpen: boolean;
}

export function SwipeHandler({ onOpenSidebar, isOpen }: SwipeHandlerProps) {
  const [touchStartX, setTouchStartX] = useState<number | null>(null);
  const swipeAreaRef = useRef<HTMLDivElement>(null);

  // The threshold distance the user needs to swipe to trigger opening
  const SWIPE_THRESHOLD = 70;
  // Width of the detection area from the left edge
  const EDGE_WIDTH = 20;

  const handleTouchStart = (e: React.TouchEvent) => {
    // Only detect touches that start near the left edge
    if (e.touches[0].clientX <= EDGE_WIDTH) {
      setTouchStartX(e.touches[0].clientX);
    }
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    if (touchStartX !== null) {
      // If we've moved far enough right, trigger the sidebar opening
      const distance = e.touches[0].clientX - touchStartX;
      if (distance > SWIPE_THRESHOLD) {
        onOpenSidebar();
        // Reset touch tracking after triggering
        setTouchStartX(null);
      }
    }
  };

  const handleTouchEnd = () => {
    // Reset touch tracking
    setTouchStartX(null);
  };

  return (
    // This invisible div covers the left edge of the screen to detect swipes
    <div
      ref={swipeAreaRef}
      className={`fixed left-0 top-16 bottom-0 w-6 z-10 ${isOpen ? "pointer-events-none" : "pointer-events-auto"}`}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
    />
  );
}

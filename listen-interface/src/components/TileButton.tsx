import React, { ReactNode } from "react";

interface TileButtonProps {
  icon: ReactNode;
  onClick: () => void;
  ariaLabel?: string;
  className?: string;
}

const TileButton: React.FC<TileButtonProps> = ({
  icon,
  onClick,
  ariaLabel,
  className = "",
}) => {
  return (
    <button
      className={`relative w-10 h-10 flex items-center justify-center ${className} transition-all duration-200`}
      onClick={onClick}
      aria-label={ariaLabel}
    >
      {/* SVG Background with gradient border */}
      <svg
        className="absolute inset-0"
        width="40"
        height="40"
        viewBox="0 0 40 40"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{ transition: "all 0.2s" }}
      >
        <rect
          width="40"
          height="40"
          rx="20"
          fill="white"
          fillOpacity="0.08"
          style={{ transition: "all 0.2s" }}
          className="group-hover:[fill-opacity:0.12]"
        />
        <rect
          x="0.5"
          y="0.5"
          width="39"
          height="39"
          rx="19.5"
          stroke="url(#paint0_linear_2039_15543)"
          strokeOpacity="0.16"
          style={{ transition: "all 0.2s" }}
          className="group-hover:[stroke-opacity:0.24]"
        />
        <defs>
          <linearGradient
            id="paint0_linear_2039_15543"
            x1="20"
            y1="40"
            x2="20"
            y2="0"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="white" stopOpacity="0" />
            <stop offset="1" stopColor="white" />
          </linearGradient>
        </defs>
      </svg>

      {/* Icon content */}
      <div className="relative z-10 text-[#D9D9D9]">{icon}</div>
    </button>
  );
};

export default TileButton;

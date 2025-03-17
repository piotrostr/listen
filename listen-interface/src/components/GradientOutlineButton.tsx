import { useMobile } from "../contexts/MobileContext";
import { VectorArrow } from "./VectorArrow";

export const GradientOutlineButton = ({
  text,
  arrow,
  onClick,
  disabled,
}: {
  text: string;
  arrow?: boolean;
  onClick?: () => void;
  disabled?: boolean;
}) => {
  const { isMobile, isVerySmallScreen } = useMobile();

  return (
    <div
      className={`relative flex ${isMobile ? "w-full" : "w-[358px]"} ${isVerySmallScreen ? "h-[50px]" : "h-[56px]"} justify-center items-center`}
    >
      <div className="absolute inset-0">
        <svg
          className="w-full h-full"
          preserveAspectRatio="none"
          viewBox="0 0 358 56"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            width="100%"
            height="100%"
            rx="12"
            fill="url(#paint0_linear_2033_12189)"
          />
          <defs>
            <linearGradient
              id="paint0_linear_2033_12189"
              x1="0%"
              y1="50%"
              x2="100%"
              y2="50%"
              gradientUnits="userSpaceOnUse"
            >
              <stop stopColor="#FD98A2" />
              <stop offset="0.315" stopColor="#FB2671" />
              <stop offset="0.675" stopColor="#A42CCD" />
              <stop offset="1" stopColor="#7F4AFB" />
            </linearGradient>
          </defs>
        </svg>
      </div>
      <button
        className={`box-border flex flex-row justify-center items-center gap-1 ${isVerySmallScreen ? "py-4" : "py-5"} ${isVerySmallScreen ? "px-4" : "px-5"} ${isVerySmallScreen ? "h-[46px]" : "h-[52px]"} bg-[#151518] rounded-xl relative z-10 m-[2px] ${
          isMobile ? "w-[calc(100%-4px)]" : "w-[354px]"
        }`}
        onClick={onClick}
        disabled={disabled}
      >
        <p
          className={`font-['Space_Grotesk'] font-normal ${isVerySmallScreen ? "text-[14px] leading-[14px]" : "text-[16px] leading-[16px]"} text-white text-center`}
        >
          {text}
        </p>
        {arrow && <VectorArrow />}
      </button>
    </div>
  );
};

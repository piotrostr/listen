import { useMobile } from "../contexts/MobileContext";
import { VectorArrow } from "./VectorArrow";

export const OutlineButton = ({
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
      className={`relative flex ${isMobile ? "w-full" : "w-[358px]"} justify-center items-center`}
    >
      <button
        className={`box-border flex flex-row justify-center items-center gap-${isVerySmallScreen ? "3" : "4"} ${isVerySmallScreen ? "py-4" : "py-5"} ${isVerySmallScreen ? "px-4" : "px-5"} ${isVerySmallScreen ? "h-[50px]" : "h-[56px]"} bg-[#151518] border border-[#2D2D2D] rounded-xl ${
          isMobile ? "w-full" : "w-[358px]"
        }`}
        onClick={onClick}
        disabled={disabled}
      >
        <p
          className={`font-['Space_Grotesk'] font-normal ${isVerySmallScreen ? "text-[14px] leading-[14px]" : "text-[16px] leading-[16px]"} text-white flex-grow text-center`}
        >
          {text}
        </p>
        {arrow && <VectorArrow />}
      </button>
    </div>
  );
};

import { memo } from "react";

export const BurgerIcon = memo(function BurgerIcon({
  isOpen,
  onClick,
}: {
  isOpen: boolean;
  onClick: () => void;
}) {
  return (
    <button onClick={onClick} className="focus:outline-none">
      {isOpen ? (
        <svg
          width="40"
          height="40"
          viewBox="0 0 40 40"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            width="40"
            height="40"
            rx="20"
            fill="white"
            fillOpacity="0.08"
          />
          <rect
            x="0.5"
            y="0.5"
            width="39"
            height="39"
            rx="19.5"
            stroke="url(#paint0_linear_2039_14182)"
            strokeOpacity="0.16"
          />
          <path
            d="M21.0607 20L24.0295 17.0312C24.1704 16.8906 24.2496 16.6997 24.2498 16.5007C24.25 16.3016 24.171 16.1106 24.0304 15.9697C23.8897 15.8288 23.6989 15.7495 23.4998 15.7494C23.3007 15.7492 23.1097 15.8281 22.9688 15.9687L20.0001 18.9375L17.0313 15.9687C16.8904 15.8278 16.6993 15.7487 16.5001 15.7487C16.3008 15.7487 16.1097 15.8278 15.9688 15.9687C15.8279 16.1096 15.7488 16.3007 15.7488 16.5C15.7488 16.6992 15.8279 16.8903 15.9688 17.0312L18.9376 20L15.9688 22.9687C15.8279 23.1096 15.7488 23.3007 15.7488 23.5C15.7488 23.6992 15.8279 23.8903 15.9688 24.0312C16.1097 24.1721 16.3008 24.2513 16.5001 24.2513C16.6993 24.2513 16.8904 24.1721 17.0313 24.0312L20.0001 21.0625L22.9688 24.0312C23.1097 24.1721 23.3008 24.2513 23.5001 24.2513C23.6993 24.2513 23.8904 24.1721 24.0313 24.0312C24.1722 23.8903 24.2514 23.6992 24.2514 23.5C24.2514 23.3007 24.1722 23.1096 24.0313 22.9687L21.0607 20Z"
            fill="#D9D9D9"
          />
          <defs>
            <linearGradient
              id="paint0_linear_2039_14182"
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
      ) : (
        <svg
          width="40"
          height="40"
          viewBox="0 0 40 40"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            width="40"
            height="40"
            rx="20"
            fill="white"
            fillOpacity="0.08"
          />
          <rect
            x="0.5"
            y="0.5"
            width="39"
            height="39"
            rx="19.5"
            stroke="url(#paint0_linear_2039_14744)"
            strokeOpacity="0.16"
          />
          <circle cx="13" cy="20" r="2.5" fill="#D9D9D9" />
          <circle cx="20" cy="20" r="2.5" fill="#D9D9D9" />
          <circle cx="27" cy="20" r="2.5" fill="#D9D9D9" />
          <defs>
            <linearGradient
              id="paint0_linear_2039_14744"
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
      )}
    </button>
  );
});

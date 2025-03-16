import { useTranslation } from "react-i18next";
import { useMobile } from "../contexts/MobileContext";
import { BetaWarning } from "./BetaWarning";
import { VersionDisplay } from "./VersionAndLanguage";

const VectorArrow = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 16 16"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M2.34315 7.375C1.99797 7.375 1.71815 7.65482 1.71815 8C1.71815 8.34518 1.99797 8.625 2.34315 8.625L2.34315 7.375ZM14.0988 8.44194C14.3429 8.19786 14.3429 7.80213 14.0988 7.55806L10.1213 3.58058C9.87724 3.3365 9.48151 3.3365 9.23744 3.58058C8.99336 3.82466 8.99336 4.22039 9.23744 4.46447L12.773 8L9.23744 11.5355C8.99336 11.7796 8.99336 12.1753 9.23744 12.4194C9.48152 12.6635 9.87724 12.6635 10.1213 12.4194L14.0988 8.44194ZM2.34315 8.625L13.6569 8.625L13.6569 7.375L2.34315 7.375L2.34315 8.625Z"
      fill="#D9D9D9"
    />
  </svg>
);

const OutlineButton = ({ text, arrow }: { text: string; arrow?: boolean }) => {
  const { isMobile } = useMobile();
  return (
    <div
      className={`relative flex ${isMobile ? "w-full" : "w-[358px]"} justify-center items-center`}
    >
      <button
        className={`box-border flex flex-row justify-center items-center gap-4 py-5 px-5 h-[56px] bg-[#151518] border border-[#2D2D2D] rounded-xl ${
          isMobile ? "w-full" : "w-[358px]"
        }`}
      >
        <p className="font-['Space_Grotesk'] font-normal text-[16px] leading-[16px] text-white flex-grow text-center">
          {text}
        </p>
        {arrow && <VectorArrow />}
      </button>
    </div>
  );
};

const GradientOutlineButton = ({
  text,
  arrow,
}: {
  text: string;
  arrow?: boolean;
}) => {
  const { isMobile } = useMobile();
  return (
    <div
      className={`relative flex ${isMobile ? "w-full" : "w-[358px]"} h-[56px] justify-center items-center`}
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
        className={`box-border flex flex-row justify-center items-center gap-1 py-5 px-5 h-[52px] bg-[#151518] rounded-xl relative z-10 m-[2px] ${
          isMobile ? "w-[calc(100%-4px)]" : "w-[354px]"
        }`}
      >
        <p className="font-['Space_Grotesk'] font-normal text-[16px] leading-[16px] text-white text-center">
          {text}
        </p>
        {arrow && <VectorArrow />}
      </button>
    </div>
  );
};

export function GettingStarted() {
  const { t } = useTranslation();
  const { isMobile } = useMobile();

  return (
    <div className="flex flex-col items-center gap-4 p-2 w-full h-full overflow-hidden justify-between">
      <div
        className={`w-full max-w-2xl flex flex-col ${isMobile ? "items-start" : "items-center"} ${isMobile ? "text-left" : "text-center"} gap-2 p-2`}
      >
        <h2 className="font-light text-[28px] leading-[40px] tracking-[-0.03em] mb-2">
          {t("getting_started.listen_hi")}
        </h2>
        <p className="font-light text-[28px] leading-[40px] tracking-[-0.03em]">
          {t("getting_started.listen_intro")}
        </p>
      </div>
      <div>
        <p className="font-[500] text-[32px] leading-[40px] tracking-[-0.04em]">
          {t("getting_started.where_should_we_start")}
        </p>
      </div>
      <div className="flex flex-col gap-2 w-full">
        <GradientOutlineButton
          text={t("getting_started.lets_make_a_trade")}
          arrow={true}
        />
        <OutlineButton
          text={t("getting_started.create_an_automated_strategy")}
        />
        <OutlineButton text={t("getting_started.run_some_research")} />
        <OutlineButton text={t("getting_started.skip")} />
      </div>
      <div className="flex flex-col gap-2 w-full text-center text-xs">
        <BetaWarning />
        <VersionDisplay />
      </div>
    </div>
  );
}

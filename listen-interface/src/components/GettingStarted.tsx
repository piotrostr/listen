import { useTranslation } from "react-i18next";
import { useMobile } from "../contexts/MobileContext";
import { BetaWarning } from "./BetaWarning";

const OutlineButton = ({ text }: { text: string }) => {
  return (
    <div className="relative inline-block">
      <button className="relative z-10 px-6 py-3 rounded-2xl bg-[#151518] border border-white border-opacity-50">
        <p className="font-['Space_Grotesk'] font-normal text-[18px] leading-[16px] text-white">
          {text}
        </p>
      </button>
    </div>
  );
};

const GradientOutlineButton = ({ text }: { text: string }) => {
  const { isMobile } = useMobile();
  return (
    <div
      className={`relative flex h-[70px] ${isMobile ? "w-full" : ""} justify-center items-center`}
    >
      <button
        className={`relative z-10 px-6 py-3 rounded-2xl bg-transparent border-2 border-transparent ${isMobile ? "w-full" : ""}`}
      >
        <p className="font-['Space_Grotesk'] font-weight-400 text-[18px] leading-[16px] text-white">
          {text}
        </p>
      </button>
      <div className="absolute inset-0">
        <svg
          className="w-full h-full"
          preserveAspectRatio="none"
          viewBox="0 0 358 70"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            width="100%"
            height="100%"
            rx="16"
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
    </div>
  );
};

export function GettingStarted() {
  const { t } = useTranslation();
  const { isMobile } = useMobile();

  return (
    <div className="flex flex-col items-center gap-4 p-2 w-full overflow-hidden">
      <div
        className={`w-full max-w-2xl flex flex-col items-center ${isMobile ? "text-left" : "text-center"} gap-2 p-2`}
      >
        <h2 className="font-light text-[28px] leading-[40px] tracking-[-0.03em] mt-5 mb-2">
          {t("getting_started.listen_hi")}
        </h2>
        <p className="font-light text-[28px] leading-[40px] tracking-[-0.03em]">
          {t("getting_started.listen_intro")}
        </p>
        <p className="font-bold text-[28px] leading-[40px] tracking-[-0.03em]">
          {t("getting_started.where_should_we_start")}
        </p>
      </div>
      <GradientOutlineButton text={t("getting_started.lets_make_a_trade")} />
      <OutlineButton text={t("getting_started.create_an_automated_strategy")} />
      <OutlineButton text={t("getting_started.run_some_research")} />

      <BetaWarning />
    </div>
  );
}

import { useTranslation } from "react-i18next";
import { BetaWarning } from "./BetaWarning";

const OutlineButton = ({ children }: { children: React.ReactNode }) => {
  return <button>{children}</button>;
};

const GradientOutlineButton = ({ text }: { text: string }) => {
  return (
    <div className="relative inline-block">
      <button className="relative z-10 px-6 py-3 rounded-2xl bg-transparent border-2 border-transparent">
        <p className="text-sm lg:text-base text-white">{text}</p>
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

  return (
    <div className="flex flex-col items-center gap-4 p-2 w-full overflow-hidden">
      <div className="w-full max-w-2xl mx-auto flex flex-col items-center text-center gap-2">
        <h2 className="text-xl lg:text-2xl font-bold mt-5 mb-2">
          {t("getting_started.listen_hi")}
        </h2>
        <p className="text-sm lg:text-base">
          {t("getting_started.listen_intro")}
        </p>
        <p className="text-sm lg:text-base">
          {t("getting_started.where_should_we_start")}
        </p>
      </div>
      <GradientOutlineButton text={t("getting_started.lets_make_a_trade")} />
      <BetaWarning />
    </div>
  );
}

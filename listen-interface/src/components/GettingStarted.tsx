import { useTranslation } from "react-i18next";
import { BetaWarning } from "./BetaWarning";

const OutlineButton = ({ children }: { children: React.ReactNode }) => {
  return <button>{children}</button>;
};

const GradientOutlineButton = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="relative">
      {children}
      <div className="absolute inset-0">
        <svg
          width="358"
          height="70"
          viewBox="0 0 358 70"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            width="358"
            height="70"
            rx="16"
            fill="url(#paint0_linear_2033_12189)"
          />
          <defs>
            <linearGradient
              id="paint0_linear_2033_12189"
              x1="1.20047e-05"
              y1="40"
              x2="358"
              y2="35"
              gradientUnits="userSpaceOnUse"
            >
              <stop stop-color="#FD98A2" />
              <stop offset="0.315" stop-color="#FB2671" />
              <stop offset="0.675" stop-color="#A42CCD" />
              <stop offset="1" stop-color="#7F4AFB" />
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
      <GradientOutlineButton>
        <p className="text-sm lg:text-base text-white">
          {t("getting_started.lets_make_a_trade")}
        </p>
      </GradientOutlineButton>
      <BetaWarning />
    </div>
  );
}

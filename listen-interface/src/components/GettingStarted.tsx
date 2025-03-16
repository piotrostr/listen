import { usePrivy } from "@privy-io/react-auth";
import { useTranslation } from "react-i18next";
import { useMobile } from "../contexts/MobileContext";
import { BetaWarning } from "./BetaWarning";
import { GradientOutlineButton } from "./GradientOutlineButton";
import { OutlineButton } from "./OutlineButton";
import { VersionDisplay } from "./VersionAndLanguage";

export function GettingStarted() {
  const { t } = useTranslation();
  const { isMobile, isVerySmallScreen } = useMobile();
  const { login, ready } = usePrivy();

  return (
    <div
      className={`flex flex-col items-center ${isVerySmallScreen ? "gap-3" : "gap-4"} ${isVerySmallScreen ? "p-1.5" : "p-2"} w-full h-full overflow-hidden justify-between`}
    >
      <div
        className={`w-full max-w-2xl flex flex-col ${isMobile ? "items-start" : "items-center"} ${isMobile ? "text-left" : "text-center"} ${isVerySmallScreen ? "gap-1.5" : "gap-2"} ${isVerySmallScreen ? "p-1.5" : "p-2"}`}
      >
        <h2
          className={`font-light ${isVerySmallScreen ? "text-[26px] leading-[38px]" : "text-[28px] leading-[40px]"} tracking-[-0.03em] ${isVerySmallScreen ? "mb-1.5" : "mb-2"}`}
        >
          {t("getting_started.listen_hi")}
        </h2>
        <p
          className={`font-light ${isVerySmallScreen ? "text-[26px] leading-[38px]" : "text-[28px] leading-[40px]"} tracking-[-0.03em]`}
        >
          {t("getting_started.listen_intro")}
        </p>
      </div>
      {isMobile ? (
        <>
          <div>
            <p
              className={`font-[500] ${isVerySmallScreen ? "text-[28px] leading-[36px]" : "text-[32px] leading-[40px]"} tracking-[-0.04em]`}
            >
              {t("getting_started.where_should_we_start")}
            </p>
          </div>
          <div
            className={`flex flex-col ${isVerySmallScreen ? "gap-1.5" : "gap-2"} w-full`}
          >
            <GradientOutlineButton
              text={t("getting_started.lets_make_a_trade")}
              arrow={true}
              onClick={login}
              disabled={!ready}
            />
            <OutlineButton
              text={t("getting_started.create_an_automated_strategy")}
            />
            <OutlineButton text={t("getting_started.run_some_research")} />
            <OutlineButton text={t("getting_started.skip")} />
          </div>
          <div
            className={`flex flex-col ${isVerySmallScreen ? "gap-1.5" : "gap-2"} w-full text-center text-xs justify-center items-center mb-1`}
          >
            <BetaWarning />
            <VersionDisplay />
          </div>
        </>
      ) : (
        <>
          <GradientOutlineButton
            text={t("getting_started.get_started")}
            arrow={true}
            onClick={login}
            disabled={!ready}
          />
          <div
            className={`flex flex-col ${isVerySmallScreen ? "gap-1.5" : "gap-2"} w-full text-center text-xs justify-center items-center mb-1`}
          >
            <BetaWarning />
            <VersionDisplay />
          </div>
        </>
      )}
    </div>
  );
}

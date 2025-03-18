import { useGuestAccounts, usePrivy } from "@privy-io/react-auth";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useMobile } from "../contexts/MobileContext";
import { BetaWarning } from "./BetaWarning";
import { FullPageLoading } from "./FullPageLoading";
import { GradientOutlineButton } from "./GradientOutlineButton";
import { OutlineButton } from "./OutlineButton";
import { VersionDisplay } from "./VersionAndLanguage";

export function GettingStarted() {
  const { t } = useTranslation();
  const { isMobile, isVerySmallScreen } = useMobile();
  const { ready, login } = usePrivy();
  const { createGuestAccount } = useGuestAccounts();
  const [isCreatingGuestAccount, setIsCreatingGuestAccount] = useState(false);
  const navigate = useNavigate();

  const handleContinue = async (prompt?: string) => {
    try {
      setIsCreatingGuestAccount(true);
      await createGuestAccount();
      setIsCreatingGuestAccount(false);
      if (prompt) {
        await navigate({
          to: "/",
          search: {
            message: prompt,
            new: true,
          },
        });
      } else {
        await navigate({
          to: "/",
        });
      }
    } catch (error) {
      console.error("Error creating guest account:", error);
      setIsCreatingGuestAccount(false);
    }
  };

  if (isCreatingGuestAccount) {
    return <FullPageLoading />;
  }

  return (
    <div
      className={`flex flex-col items-center ${isVerySmallScreen ? "gap-3" : "gap-4"} ${isVerySmallScreen ? "p-1.5" : "p-2"} w-full h-full overflow-hidden ${isMobile ? "justify-between" : "justify-center"}`}
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
      <div>
        <p
          className={`font-[500] ${isVerySmallScreen ? "text-[28px] leading-[36px]" : "text-[32px] leading-[40px]"} tracking-[-0.04em]`}
        >
          {t("getting_started.where_should_we_start")}
        </p>
      </div>
      <div
        className={`flex flex-col ${isVerySmallScreen ? "gap-1.5" : "gap-2"} w-full justify-center items-center`}
      >
        <GradientOutlineButton
          text={t("getting_started.lets_make_a_trade")}
          arrow={true}
          onClick={() => handleContinue(t("getting_started.lets_make_a_trade"))}
          disabled={!ready || isCreatingGuestAccount}
        />
        <OutlineButton
          text={t("getting_started.create_an_automated_strategy")}
          onClick={() =>
            handleContinue(t("getting_started.create_an_automated_strategy"))
          }
          disabled={!ready || isCreatingGuestAccount}
        />
        <OutlineButton
          text={t("getting_started.run_some_research")}
          onClick={() => handleContinue(t("getting_started.run_some_research"))}
          disabled={!ready || isCreatingGuestAccount}
        />
        <OutlineButton
          text={t("getting_started.login")}
          onClick={() => login()}
          disabled={!ready || isCreatingGuestAccount}
        />
      </div>
      <div
        className={`flex flex-col ${isVerySmallScreen ? "gap-1.5" : "gap-2"} w-full text-center text-xs justify-center items-center mb-1`}
      >
        <BetaWarning />
        <VersionDisplay />
      </div>
    </div>
  );
}

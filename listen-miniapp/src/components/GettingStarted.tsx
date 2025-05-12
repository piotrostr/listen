import { useGuestAccounts, usePrivy } from "@privy-io/react-auth";
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { worldchainEnabled } from "../config/env";
import { useMobile } from "../contexts/MobileContext";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";
import { useWorldAuth } from "../hooks/useWorldLogin";
import { FullPageLoading } from "./FullPageLoading";
import { GradientOutlineButton } from "./GradientOutlineButton";
import { VersionDisplay } from "./VersionAndLanguage";

export function GettingStarted() {
  const { t } = useTranslation();
  const { isMobile, isVerySmallScreen } = useMobile();
  const { login } = usePrivy();
  const { createGuestAccount } = useGuestAccounts();
  const [isCreatingGuestAccount, setIsCreatingGuestAccount] = useState(false);
  const { worldLogin, isLoading: isWorldLoading } = useWorldAuth();
  const { isLoading } = useIsAuthenticated();
  const navigate = useNavigate();

  const handleLogin = async () => {
    if (worldchainEnabled) {
      await worldLogin();
      setIsCreatingGuestAccount(true);
      await createGuestAccount();
      setIsCreatingGuestAccount(false);
      await navigate({
        to: "/",
      });
    } else {
      await login();
    }
  };

  if (isCreatingGuestAccount || isLoading) {
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
      <div
        className={`flex flex-col ${isVerySmallScreen ? "gap-1.5" : "gap-2"} w-full justify-center items-center`}
      ></div>
      <div
        className={`flex flex-col ${isVerySmallScreen ? "gap-3" : "gap-4"} w-full text-center text-xs justify-center items-center mb-2`}
      >
        <GradientOutlineButton
          arrow={true}
          text={"Sign In"}
          onClick={handleLogin}
          disabled={isCreatingGuestAccount || isWorldLoading}
        />
        <VersionDisplay />
      </div>
    </div>
  );
}

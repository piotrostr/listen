import { usePrivy } from "@privy-io/react-auth";
import { useNavigate } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { useMobile } from "../contexts/MobileContext";
import { GradientOutlineButton } from "./GradientOutlineButton";
import { VersionDisplay } from "./VersionAndLanguage";

export function GettingStarted() {
  const { t } = useTranslation();
  const { isMobile, isVerySmallScreen } = useMobile();
  const { ready, login } = usePrivy();
  const navigate = useNavigate();

  const handleConnectWallet = async () => {
    // Privy's login() will show wallet connection modal
    await login();
    // After successful connection, navigate to home
    await navigate({ to: "/" });
  };

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
          arrow={true}
          text="Connect Wallet"
          onClick={handleConnectWallet}
          disabled={!ready}
        />
        <p className="text-gray-400 text-sm text-center max-w-md">
          Connect your wallet to start trading, view your portfolio, and interact
          with the AI assistant.
        </p>
      </div>
      <div
        className={`flex flex-col ${isVerySmallScreen ? "gap-1.5" : "gap-2"} w-full text-center text-xs justify-center items-center mb-1`}
      >
        <VersionDisplay />
      </div>
    </div>
  );
}

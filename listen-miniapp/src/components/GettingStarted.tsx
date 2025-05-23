import { usePrivy } from "@privy-io/react-auth";
import { useNavigate } from "@tanstack/react-router";
import { MiniKit } from "@worldcoin/minikit-js";
import { useTranslation } from "react-i18next";
import { worldchainEnabled } from "../config/env";
import { useMobile } from "../contexts/MobileContext";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";
import { useWorldAuth } from "../hooks/useWorldLogin";
import { FullPageLoading } from "./FullPageLoading";
import { GradientOutlineButton } from "./GradientOutlineButton";

export function GettingStarted() {
  const { t } = useTranslation();
  const { isMobile, isVerySmallScreen } = useMobile();
  const { login } = usePrivy();
  const { worldLogin, isLoading: isWorldLoading } = useWorldAuth();
  const { isLoading } = useIsAuthenticated();
  const navigate = useNavigate();

  // Check if MiniKit is available
  const isMiniKitAvailable = (() => {
    try {
      return MiniKit.isInstalled();
    } catch {
      return false;
    }
  })();

  const handleLogin = async () => {
    try {
      if (worldchainEnabled) {
        await worldLogin();
        await navigate({
          to: "/",
        });
      } else {
        await login();
      }
    } catch (error) {
      console.error("Login error:", error);
    }
  };

  if (isLoading) {
    return <FullPageLoading />;
  }

  return (
    <div
      className={`flex flex-col items-center ${isVerySmallScreen ? "gap-3" : "gap-4"} ${isVerySmallScreen ? "p-1.5" : "p-2"} w-full h-full overflow-hidden ${isMobile ? "justify-between" : "justify-center"}`}
    >
      <div
        className={`w-full max-w-2xl flex flex-col ${isMobile ? "items-start" : "items-center"} ${isMobile ? "text-left" : "text-center"} ${isVerySmallScreen ? "gap-1.5" : "gap-2"} ${isVerySmallScreen ? "p-1.5" : "p-2"}`}
      >
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
        {worldchainEnabled && !isMiniKitAvailable && (
          <div className="text-orange-500 text-sm mb-2 text-center max-w-md">
            To use Worldcoin authentication, please open this app in the World
            App.
            <br />
            <span className="text-xs text-gray-400">
              Currently running in:{" "}
              {navigator.userAgent.includes("Chrome")
                ? "Chrome"
                : navigator.userAgent.includes("Safari")
                  ? "Safari"
                  : navigator.userAgent.includes("Firefox")
                    ? "Firefox"
                    : "Browser"}
            </span>
            <br />
            <a
              href={`${window.location.origin}${window.location.pathname}?test-worldcoin=true`}
              className="text-blue-400 hover:text-blue-300 underline mt-2 inline-block"
            >
              Or click here to test with fallback authentication
            </a>
          </div>
        )}
        {process.env.NODE_ENV === "development" && worldchainEnabled && (
          <div className="text-yellow-500 text-xs mb-2">
            Development mode: Using fallback authentication
          </div>
        )}
        {window.location.search.includes("test-worldcoin=true") && (
          <div className="text-blue-500 text-xs mb-2">
            Testing mode: Using fallback authentication
          </div>
        )}
        <GradientOutlineButton
          arrow={true}
          text={isWorldLoading ? "Signing In..." : "Sign In"}
          onClick={handleLogin}
          disabled={
            isWorldLoading ||
            (worldchainEnabled &&
              !isMiniKitAvailable &&
              !window.location.search.includes("test-worldcoin=true"))
          }
        />
        {worldchainEnabled &&
          !isMiniKitAvailable &&
          !window.location.search.includes("test-worldcoin=true") && (
            <p className="text-xs text-gray-500 mt-2">
              Button disabled - World App required for Worldcoin authentication
            </p>
          )}
      </div>
    </div>
  );
}

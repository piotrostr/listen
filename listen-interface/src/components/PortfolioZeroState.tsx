import { useNavigate } from "@tanstack/react-router";
import { usePanel } from "../contexts/PanelContext";
import { useMobile } from "../contexts/MobileContext";

export function PortfolioZeroState() {
  const navigate = useNavigate();
  const { setActivePanel } = usePanel();
  const { isMobile } = useMobile();

  const handleGetStarted = () => {
    const onboardingPrompt = "Hey Listen, let's get me onboarded with Solana and EVM wallets creation, also give me a brief rundown of the things we can do afterwards!";
    
    // Navigate to chat with the onboarding message
    navigate({
      to: "/",
      search: {
        message: onboardingPrompt,
        new: true,
      },
    });

    // On mobile, close the portfolio panel to show chat
    if (isMobile) {
      setActivePanel(null);
    }
  };

  return (
    <div className="flex flex-col items-center h-full px-6 text-center">
      <div className="mb-8 my-[40%]">
        <img
          src="/listen-galaxy.png"
          alt="Listen Galaxy Illustration"
          className="w-64 h-64 mx-auto rounded-2xl"
        />
      </div>

      {/* Main content */}
      <div className="max-w-sm mx-auto mb-8">
        <h2
          className="text-white mb-3 text-center"
          style={{
            fontFamily: "Space Grotesk",
            fontWeight: 500,
            fontSize: "32px",
            lineHeight: "130%",
            letterSpacing: "-4%",
          }}
        >
          Say it. Trade it.
        </h2>
        <p
          className="text-gray-400 text-center"
          style={{
            fontFamily: "Space Grotesk",
            fontWeight: 400,
            fontSize: "18px",
            lineHeight: "140%",
            letterSpacing: "-3%",
          }}
        >
          You haven't created a wallet yet. Ask Listen to get you through
          onboarding!
        </p>
      </div>

      {/* Action button */}
      <div className="w-full max-w-sm">
        <button
          className="text-white font-semibold transition-colors hover:opacity-90"
          style={{
            display: "flex",
            flexDirection: "row",
            justifyContent: "center",
            alignItems: "center",
            padding: "14px 16px",
            gap: "6px",
            margin: "0 auto",
            width: "100%",
            maxWidth: "358px",
            height: "56px",
            background:
              "linear-gradient(44.8deg, #FD98A2 -6.27%, #FB2671 36.3%, #A42CCD 84.95%, #7F4AFB 128.87%)",
            borderRadius: "9999px",
            border: "none",
            cursor: "pointer",
          }}
          onClick={handleGetStarted}
        >
          Get Started
        </button>
      </div>
    </div>
  );
}

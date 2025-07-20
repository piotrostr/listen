export function PortfolioZeroState() {
  return (
    <div className="flex flex-col items-center justify-center h-full px-6 text-center">
      <div className="mb-8">
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
    </div>
  );
}

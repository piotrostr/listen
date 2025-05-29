import { useWalletStore } from "../store/walletStore";

export const ChainSwitcher = () => {
  const { setActiveWallet, activeWallet } = useWalletStore();

  const handleChainSelect = () => {
    if (activeWallet === "worldchain") {
      setActiveWallet("listen");
    } else {
      setActiveWallet("worldchain");
    }
  };

  return (
    <div className="chain-switcher w-[234px] h-[46px]">
      <svg
        width="234"
        height="46"
        viewBox="0 0 234 46"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className="w-full h-auto"
      >
        <defs>
          <linearGradient
            id="paint0_linear_4181_3195"
            x1="4"
            y1="46.75"
            x2="33.9064"
            y2="-45.9601"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="#FD98A2" />
            <stop offset="0.315" stopColor="#FB2671" />
            <stop offset="0.675" stopColor="#A42CCD" />
            <stop offset="1" stopColor="#7F4AFB" />
          </linearGradient>
        </defs>

        {/* Background */}
        <rect width="234" height="46" rx="23" fill="#1D1D21" />

        {/* World Chain Button */}
        <g
          onClick={handleChainSelect}
          className="cursor-pointer"
          style={{ opacity: activeWallet === "worldchain" ? 1 : 0.7 }}
        >
          {activeWallet === "worldchain" ? (
            <>
              <rect
                x="4.5"
                y="4.5"
                width="116"
                height="37"
                rx="18.5"
                fill="#151518"
              />
              <rect
                x="4.5"
                y="4.5"
                width="116"
                height="37"
                rx="18.5"
                stroke="url(#paint0_linear_4181_3195)"
              />
            </>
          ) : (
            <rect
              x="4.5"
              y="4.5"
              width="116"
              height="37"
              rx="18.5"
              fill="transparent"
            />
          )}

          <text
            x="62.5"
            y="23"
            textAnchor="middle"
            dominantBaseline="middle"
            fill="white"
            fontSize="12"
            fontFamily="system-ui, -apple-system, sans-serif"
            fontWeight="500"
          >
            Worldchain
          </text>
        </g>

        {/* All Chains Button */}
        <g
          onClick={handleChainSelect}
          className="cursor-pointer"
          style={{ opacity: activeWallet === "listen" ? 1 : 0.7 }}
        >
          {activeWallet === "listen" ? (
            <>
              <rect
                x="113.5"
                y="4.5"
                width="116"
                height="37"
                rx="18.5"
                fill="#151518"
              />
              <rect
                x="113.5"
                y="4.5"
                width="116"
                height="37"
                rx="18.5"
                stroke="url(#paint0_linear_4181_3195)"
              />
            </>
          ) : (
            <rect
              x="113.5"
              y="4.5"
              width="116"
              height="37"
              rx="18.5"
              fill="transparent"
            />
          )}

          <text
            x="171.5"
            y="23"
            textAnchor="middle"
            dominantBaseline="middle"
            fill={activeWallet === "listen" ? "white" : "#8F8F8F"}
            fontSize="12"
            fontFamily="system-ui, -apple-system, sans-serif"
            fontWeight="500"
          >
            All chains
          </text>
        </g>
      </svg>
    </div>
  );
};

export default ChainSwitcher;

import { useMfaEnrollment, usePrivy } from "@privy-io/react-auth";
import { useFundWallet } from "@privy-io/react-auth/solana";
import { MdOutlineArrowOutward } from "react-icons/md";
import { TbDots, TbPlus } from "react-icons/tb";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";
import TileButton from "./TileButton";

interface PortfolioSummaryProps {
  totalBalance: number;
}

const PnLArrow = ({ isPositive }: { isPositive: boolean }) => {
  const color = isPositive ? "#8DFC63" : "#FF5C5C";
  const rotation = isPositive ? 0 : 180;

  return (
    <svg
      width="15"
      height="14"
      viewBox="0 0 15 14"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{ transform: `rotate(${rotation}deg)` }}
      className="inline-block mr-1"
    >
      <path
        d="M10.7083 3.79163L4 10.5"
        stroke={color}
        strokeWidth="1.25"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M5.1665 3.5H10.9998V9.33333"
        stroke={color}
        strokeWidth="1.25"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
};

export function PortfolioSummary({ totalBalance }: PortfolioSummaryProps) {
  const { solanaAddress, activeWallet } = useWalletStore();
  const { fundWallet } = useFundWallet();
  const { login } = usePrivy();
  const { showMfaEnrollmentModal } = useMfaEnrollment();
  const portfolioPnL = usePortfolioStore((state) => state.getPortfolioPnL());
  const pnlAmount = (totalBalance * portfolioPnL) / 100;

  const handleTopupListen = async () => {
    if (solanaAddress) {
      await fundWallet(solanaAddress, { defaultFundingMethod: "card" });
    } else {
      login();
    }
  };

  const pnlColor = portfolioPnL >= 0 ? "text-[#8DFC63]" : "text-[#FF5C5C]";
  const pnlSign = portfolioPnL >= 0 ? "+" : "";

  return (
    <div className="flex flex-col justify-center p-10 gap-7 w-full rounded-[20px] pt-12 mb-2">
      <div className="text-center">
        <span className="font-space-grotesk font-medium text-[45px] leading-4 text-white">
          $
          {totalBalance.toLocaleString(undefined, {
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
          })}
        </span>
        <div
          className={`mt-4 text-lg ${pnlColor} flex items-center justify-center gap-1 font-dm-sans`}
        >
          <PnLArrow isPositive={portfolioPnL >= 0} />
          <span>
            {pnlSign}${Math.abs(pnlAmount).toFixed(2)} {pnlSign}(
            {Math.abs(portfolioPnL).toFixed(2)}%)
          </span>
        </div>
      </div>
      <div className="flex flex-row items-center gap-3 justify-center mt-2">
        <>
          {activeWallet === "listen" && (
            <>
              <TileButton
                icon={<TbPlus className="w-4 h-4" />}
                onClick={handleTopupListen}
                ariaLabel="Deposit"
              />
              <TileButton
                icon={<MdOutlineArrowOutward />}
                onClick={() => {}}
                ariaLabel="Export"
              />
              <TileButton
                icon={<TbDots className="w-5 h-5" />}
                onClick={showMfaEnrollmentModal}
                ariaLabel="More"
              />
            </>
          )}
        </>
      </div>
    </div>
  );
}

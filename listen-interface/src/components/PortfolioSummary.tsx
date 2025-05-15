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
                icon={<CopyIcon />}
                onClick={() => {
                  if (solanaAddress) {
                    navigator.clipboard.writeText(solanaAddress);
                  }
                }}
                ariaLabel="Copy"
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

export const CopyIcon = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 16 16"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="M7.33333 14.6667C6.59695 14.6667 6 14.0698 6 13.3334V7.3343C6 6.59756 6.59753 6.00046 7.33427 6.00098L13.3343 6.00521C14.0703 6.00573 14.6667 6.60253 14.6667 7.33857V13.3334C14.6667 14.0698 14.0697 14.6667 13.3333 14.6667H7.33333Z"
      stroke="#D9D9D9"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M11.3335 5.61901V2.67268C11.3335 1.93666 10.7371 1.33986 10.0011 1.33934L2.66776 1.33423C1.93102 1.33372 1.3335 1.93082 1.3335 2.66756V9.99996C1.3335 10.7364 1.93045 11.3333 2.66683 11.3333H5.61921"
      stroke="#D9D9D9"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
  </svg>
);

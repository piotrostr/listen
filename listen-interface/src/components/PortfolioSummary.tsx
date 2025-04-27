import { useFundWallet, usePrivy } from "@privy-io/react-auth";
import { MdOutlineArrowOutward } from "react-icons/md";
import { TbDots, TbPlus } from "react-icons/tb";
import { useWalletStore } from "../store/walletStore";
import TileButton from "./TileButton";

interface PortfolioSummaryProps {
  totalBalance: number;
}

export function PortfolioSummary({ totalBalance }: PortfolioSummaryProps) {
  const { solanaAddress } = useWalletStore();
  const { fundWallet } = useFundWallet();
  const { login } = usePrivy();

  const handleTopup = async () => {
    if (solanaAddress) {
      await fundWallet(solanaAddress);
    } else {
      login();
    }
  };

  return (
    <div className="flex flex-col justify-center p-10 gap-7 w-full rounded-[20px] pt-12 mb-2">
      <span className="font-space-grotesk font-medium text-[45px] leading-4 text-white text-center">
        $
        {totalBalance.toLocaleString(undefined, {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        })}
      </span>
      <div className="flex flex-row items-center gap-3 justify-center mt-2">
        <>
          <TileButton
            icon={<TbPlus className="w-4 h-4" />}
            onClick={handleTopup}
            ariaLabel="Deposit"
          />
          <TileButton
            icon={<MdOutlineArrowOutward />}
            onClick={handleTopup}
            ariaLabel="Withdraw"
          />
          <TileButton
            icon={<TbDots className="w-5 h-5" />}
            onClick={() => {}}
            ariaLabel="More"
          />
        </>
      </div>
    </div>
  );
}

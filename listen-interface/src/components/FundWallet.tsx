import { usePrivy } from "@privy-io/react-auth";
import { useFundWallet } from "@privy-io/react-auth/solana";
import { useState } from "react";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { Spinner } from "./Spinner";

interface FundWalletProps {
  error?: string | null;
}

export const FundWallet = ({ error = null }: FundWalletProps) => {
  const { ready, user } = usePrivy();
  const { data: wallets } = usePrivyWallets();
  const { fundWallet } = useFundWallet();
  const [isFunding, setIsFunding] = useState(false);

  const handleFundWallet = async () => {
    if (!wallets?.solanaWallet) return;

    try {
      setIsFunding(true);
      await fundWallet(wallets.solanaWallet);
    } catch (error) {
      console.error("Error funding Solana wallet:", error);
    } finally {
      setIsFunding(false);
    }
  };

  if (!ready || !user || !wallets?.solanaWallet) {
    return (
      <div className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center justify-center">
        <Spinner />
      </div>
    );
  }

  return (
    <div>
      {error ? (
        <div className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm">
          <span className="text-red-500">Error: {error}</span>
        </div>
      ) : (
        <button
          disabled={isFunding}
          onClick={handleFundWallet}
          className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-purple-500/10"
        >
          {isFunding ? "Funding wallet..." : "Fund Wallet"}
        </button>
      )}
    </div>
  );
};

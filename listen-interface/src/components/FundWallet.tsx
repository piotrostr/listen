import { usePrivy } from "@privy-io/react-auth";
import { useFundWallet } from "@privy-io/react-auth/solana";
import { useEffect, useState } from "react";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { SolanaWalletCreation } from "./SolanaWalletCreation";
import { Spinner } from "./Spinner";

interface FundWalletProps {
  error?: string | null;
}

export const FundWallet = ({ error = null }: FundWalletProps) => {
  const { ready, user, login } = usePrivy();
  const { data: wallets } = usePrivyWallets();
  const { fundWallet } = useFundWallet();
  const [isFunding, setIsFunding] = useState(false);
  const [isLogin, setIsLogin] = useState(false);

  useEffect(() => {
    if (user) {
      setIsLogin(false);
    }
  }, [user]);

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

  const handleLogin = () => {
    try {
      setIsLogin(true);
      login();
    } catch (error) {
      console.error("Error logging in:", error);
    } finally {
      setIsLogin(false);
    }
  };

  if (!ready) {
    return (
      <div className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center justify-center">
        <Spinner />
      </div>
    );
  }

  if (!user || user.isGuest) {
    return (
      <button
        disabled={isLogin}
        onClick={handleLogin}
        className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-[#2D2D2D]"
      >
        {isLogin ? "Logging in..." : "Login"}
      </button>
    );
  }

  if (!wallets?.solanaWallet) {
    return <SolanaWalletCreation error={error} />;
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
          className="p-2 border-2 border-[#2D2D2D] rounded-lg bg-black/40 backdrop-blur-sm flex items-center px-3 text-sm hover:bg-[#2D2D2D]"
        >
          {isFunding ? "Funding wallet..." : "Fund Wallet"}
        </button>
      )}
    </div>
  );
};

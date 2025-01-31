import { usePrivy } from "@privy-io/react-auth";
import { useSolBalance } from "../hooks/useSolBalance";
import ethereumIcon from "../assets/icons/ethereum.svg";
import { useBalance, UseBalanceReturnType } from "wagmi";
import { usePrivyWallets } from "../hooks/usePrivyWallet";
import { Address } from "viem";

const Balance = ({
  solanaBalance,
  ethereumBalance,
}: {
  solanaBalance: number | undefined;
  ethereumBalance: number | undefined;
}) => {
  return (
    <div className="flex flex-row gap-1">
      <div className="flex items-center gap-2 mr-4">
        <img
          src="https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png"
          alt="SOL"
          className="w-6 h-6 rounded-full"
        />
        <span className="text-sm text-gray-300">
          {solanaBalance?.toFixed(2) || "0.00"}
        </span>
      </div>
      <div className="flex items-center gap-2 mr-4">
        <img src={ethereumIcon} alt="ETH" className="w-6 h-6 rounded-full" />
        <span className="text-sm text-gray-300">
          {ethereumBalance?.toFixed(4) || "0.0000"}
        </span>
      </div>
    </div>
  );
};

function balanceToUI(balance: UseBalanceReturnType["data"]) {
  if (!balance?.value || !balance?.decimals) return 0;
  return Number(balance?.value) / 10 ** balance?.decimals;
}

export const Header = () => {
  const { data: solanaBalance } = useSolBalance();
  const { data: wallets } = usePrivyWallets();
  const { data: ethereumBalance } = useBalance({
    address: wallets?.evmWallet as Address,
  });
  const { user, logout } = usePrivy();

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-transparent backdrop-blur border-b border-purple-500/30">
      <div className="max-w-7xl mx-auto px-4">
        <div className="flex justify-between items-center h-16">
          {/* Left side */}
          <div className="flex items-center space-x-4">
            <img
              src="/listen-more.png"
              alt="Logo"
              className="w-8 h-8 rounded"
            />
            <span className="font-bold text-xl">listen-rs</span>
          </div>

          {/* Right side */}
          <div className="flex items-center space-x-4">
            <Balance
              solanaBalance={solanaBalance}
              ethereumBalance={balanceToUI(ethereumBalance)}
            />
            {/* Documentation Link */}
            <div className="items-center space-x-4">
              <a
                href="https://docs.listen-rs.com"
                className="text-gray-300 hover:text-white"
                title="Documentation"
              >
                <svg
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  className="hover:text-white"
                >
                  <path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z" />
                </svg>
              </a>
            </div>

            {/* GitHub Link */}
            <a
              href="https://github.com/piotrostr/listen"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-300 hover:text-white"
              title="GitHub"
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="currentColor"
                className="hover:text-white"
              >
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
              </svg>
            </a>
            {user && (
              <div className="cursor-pointer" onClick={() => logout()}>
                <svg
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                    d="M9 21H5C4.46957 21 3.96086 20.7893 3.58579 20.4142C3.21071 20.0391 3 19.5304 3 19V5C3 4.46957 3.21071 3.96086 3.58579 3.58579C3.96086 3.21071 4.46957 3 5 3H9"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                  <path
                    d="M16 17L21 12L16 7"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                  <path
                    d="M21 12H9"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  />
                </svg>
              </div>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
};

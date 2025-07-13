import { HiOutlineBanknotes } from "react-icons/hi2";

const Container = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="flex flex-col items-start p-0 w-full bg-[#0d0d0e] border-[1px] border-[#1e1e21] rounded-[20px] mt-2">
      {children}
    </div>
  );
};

export function DepositUsdcDisplay({
  transactionHash,
  uiAmount,
}: {
  transactionHash: string;
  uiAmount: string;
}) {
  const cleanHash = transactionHash.replaceAll('"', "");
  const shortHash = `${cleanHash.slice(0, 6)}...${cleanHash.slice(-4)}`;

  return (
    <Container>
      <div className="flex flex-col p-4 w-full">
        <div className="flex flex-col w-full py-1">
          <div className="flex flex-row justify-between items-center">
            <div className="flex flex-col">
              <div className="flex flex-row items-center gap-2">
                <HiOutlineBanknotes className="w-4 h-4 text-pump-green" />
                <div className="font-dm-sans font-normal text-sm text-white">
                  USDC Deposit
                </div>
              </div>
            </div>
            <div className="flex flex-col items-end">
              <div className="font-dm-sans font-normal text-sm text-white">
                {parseFloat(uiAmount).toFixed(2)} USDC
              </div>
              <a
                href={`https://arbiscan.io/tx/${cleanHash}`}
                target="_blank"
                rel="noopener noreferrer"
                className="font-dm-sans font-light text-xs text-[#868686] hover:text-white transition-colors"
              >
                {shortHash}
              </a>
            </div>
          </div>
        </div>
      </div>
    </Container>
  );
}

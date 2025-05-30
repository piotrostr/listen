import { useState } from "react";
import { formatEther } from "viem";
import { useWLDBalance } from "../hooks/useWLDBalance";
import { GradientOutlineButton } from "./GradientOutlineButton";
import { PercentageButton, percentages } from "./PercentageButton";

export const FundPanel = () => {
  const [selectedPercentage, setSelectedPercentage] = useState(0);
  const { data: balance, isLoading: balanceIsLoading } = useWLDBalance();
  const [amount, setAmount] = useState("0");

  const handlePercentageClick = (percentage: number) => {
    if (!balance) return;
    const formattedBalance = Number(formatEther(balance));
    const value = (formattedBalance * percentage) / 100;
    setAmount(value.toFixed(3));
    setSelectedPercentage(percentage);
  };

  return (
    <div
      className={`h-full flex flex-col font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container`}
    >
      <div className="flex flex-col gap-4 mt-8">
        <div className="text-white font-space-grotesk text-[32px] font-[500] leading-[130%] tracking-[-0.04em] text-center align-middle mt-5">
          Ready to trade?
        </div>
        <div className="font-space-grotesk text-[16px] font-normal leading-[140%] tracking-[-0.03em] text-center align-middle text-[#B8B8B8] mb-3 px-4">
          Buy some Solana to start trading - this <br />
          will allow to swap into any token on any chain.
        </div>
      </div>

      <div className="flex flex-col items-center justify-center gap-4 px-4">
        <div className="xs:h-[30vh] h-[20vh] flex items-center justify-center">
          <input
            inputMode="decimal"
            type="number"
            value={amount}
            onChange={(e) => {
              const value = e.target.value;
              const regex = /^\d*\.?\d{0,3}$/;
              if (regex.test(value)) {
                setAmount(value);
                if (balance) {
                  const formattedBalance = Number(formatEther(balance));
                  const percentage = (Number(value) / formattedBalance) * 100;
                  setSelectedPercentage(percentage);
                }
              }
            }}
            placeholder="0"
            className="text-6xl bg-transparent outline-none font-space-grotesk w-screen text-center text-white"
          />
        </div>

        <div className="flex gap-3 justify-center mb-2 w-full">
          {percentages.map(({ value }) => (
            <PercentageButton
              key={value}
              percentage={value}
              selectedPercentage={selectedPercentage}
              onClick={() => handlePercentageClick(value)}
            />
          ))}
        </div>

        <div className="flex items-center justify-between w-full">
          <div className="text-[#B8B8B8] font-space-grotesk">Available</div>
          <div className="flex flex-row gap-2 items-center">
            <div className="text-white flex flex-col items-end justify-end">
              <p className="text-right truncate whitespace-nowrap font-space-grotesk">
                {balanceIsLoading
                  ? "-"
                  : balance
                    ? Number(formatEther(balance)).toFixed(3)
                    : "0"}{" "}
              </p>
            </div>
            <img
              src={"https://dd.dexscreener.com/ds-data/chains/worldchain.png"}
              className="w-8 h-8"
              alt="Worldcoin"
            />
          </div>
        </div>
      </div>
      <div className="mt-auto pb-4">
        <GradientOutlineButton
          text="Fund"
          onClick={() => {
            // Handle funding logic here
            console.log("Fund clicked with amount:", amount);
          }}
        />
      </div>
    </div>
  );
};

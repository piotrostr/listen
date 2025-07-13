import { IoTrendingDown, IoTrendingUp } from "react-icons/io5";
import { MarketOpenResponse } from "../lib/hype-types";

const Container = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="flex flex-col items-start p-0 w-full bg-[#0d0d0e] border-[1px] border-[#1e1e21] rounded-[20px] mt-2">
      {children}
    </div>
  );
};

const OrderRow = ({
  totalSize,
  avgPrice,
  side,
}: {
  totalSize: string;
  avgPrice: string;
  side: "buy" | "sell";
}) => {
  const size = parseFloat(totalSize);
  const isLong = side === "buy";
  const isShort = side === "sell";
  return (
    <div className="flex flex-col w-full py-1 border-b border-[#1e1e21] last:border-b-0">
      <div className="flex flex-row justify-between items-center">
        <div className="flex flex-col">
          <div className="flex flex-row items-center gap-2">
            {isLong && <IoTrendingUp className="w-4 h-4 text-pump-green" />}
            {isShort && <IoTrendingDown className="w-4 h-4 text-pump-red" />}
            <div className="font-dm-sans font-normal text-sm text-white">
              Market {isLong ? "Buy" : "Sell"}
            </div>
          </div>
        </div>
        <div className="flex flex-col items-end">
          <div className="font-dm-sans font-normal text-sm text-white">
            {size.toFixed(4)}
          </div>
          <div className="font-dm-sans font-light text-xs text-[#868686]">
            @ ${parseFloat(avgPrice).toFixed(4)}
          </div>
        </div>
      </div>
    </div>
  );
};

export function MarketOpenDisplay({
  marketOpenResponse,
  side,
}: {
  marketOpenResponse: MarketOpenResponse;
  side: "buy" | "sell";
}) {
  if (marketOpenResponse.status !== "ok") {
    return (
      <Container>
        <div className="flex flex-col p-4 w-full">
          <div className="font-dm-sans font-normal text-sm text-pump-red">
            Order failed
          </div>
        </div>
      </Container>
    );
  }

  const orderData = marketOpenResponse.response.data;

  return (
    <Container>
      <div className="flex flex-col p-4 w-full">
        <div className="flex flex-col gap-1">
          {orderData.statuses.map((status, index) => (
            <OrderRow
              key={`order-${index}`}
              totalSize={status.filled.totalSz}
              avgPrice={status.filled.avgPx}
              side={side}
            />
          ))}
        </div>
      </div>
    </Container>
  );
}

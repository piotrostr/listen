import { SciChartReact } from "scichart-react";
import { L2OrderbookSnapshot } from "../types/orderbook";
import { drawOrderbook } from "./OrderbookChart";

export const OrderbookDisplay = ({
  orderbookSnapshot,
}: {
  orderbookSnapshot: L2OrderbookSnapshot;
}) => {
  const { coin } = orderbookSnapshot;

  const initChart = (rootElement: string | HTMLDivElement) => {
    return drawOrderbook(rootElement, orderbookSnapshot);
  };

  return (
    <div className="w-full h-full mt-3">
      <h1 className="text-xl font-bold">{coin} Market Depth</h1>
      <div className="w-full h-96">
        <SciChartReact initChart={initChart} className="w-full h-full" />
      </div>
    </div>
  );
};

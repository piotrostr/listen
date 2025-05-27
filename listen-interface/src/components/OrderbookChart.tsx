import {
  EAutoRange,
  EAxisAlignment,
  FastMountainRenderableSeries,
  NumberRange,
  NumericAxis,
  SciChartSurface,
  XyDataSeries,
} from "scichart";
import { appTheme } from "../theme";
import { L2OrderbookSnapshot } from "../types/orderbook";

// Configure SciChart to load WASM files from public directory
SciChartSurface.configure({
  wasmUrl: "/scichart2d.wasm",
  dataUrl: "/scichart2d.data",
});

export const drawOrderbook = async (
  rootElement: string | HTMLDivElement,
  orderbookSnapshot: L2OrderbookSnapshot
) => {
  // Create a SciChartSurface with transparent background
  const { wasmContext, sciChartSurface } = await SciChartSurface.create(
    rootElement,
    {
      theme: {
        ...appTheme.SciChartJsTheme,
        sciChartBackground: "transparent",
        loadingAnimationBackground: "transparent",
      },
    }
  );

  const xAxis = new NumericAxis(wasmContext, {
    axisAlignment: EAxisAlignment.Top,
    labelPrecision: 4,
    rotation: 90,
    drawMajorBands: false,
    drawMinorGridLines: false,
    drawMajorGridLines: false,
    axisBorder: {
      borderTop: 0,
      borderBottom: 0,
      borderLeft: 0,
      borderRight: 0,
    },
  });

  sciChartSurface.xAxes.add(xAxis);

  const yAxis = new NumericAxis(wasmContext, {
    autoRange: EAutoRange.Always,
    growBy: new NumberRange(0, 0.05),
    drawMajorBands: false,
    drawMinorGridLines: false,
    drawMajorGridLines: false,
    axisBorder: {
      borderTop: 0,
      borderBottom: 0,
      borderLeft: 0,
      borderRight: 0,
    },
  });
  sciChartSurface.yAxes.add(yAxis);

  // Extract bids and asks from the orderbook data
  const bids = orderbookSnapshot.levels[0] || []; // bids
  const asks = orderbookSnapshot.levels[1] || []; // asks

  // Convert string prices to numbers and calculate cumulative volumes
  const bidData = bids
    .map((level) => ({
      price: parseFloat(level.px),
      volume: parseFloat(level.sz),
    }))
    .sort((a, b) => b.price - a.price); // Sort bids descending

  const askData = asks
    .map((level) => ({
      price: parseFloat(level.px),
      volume: parseFloat(level.sz),
    }))
    .sort((a, b) => a.price - b.price); // Sort asks ascending

  // Calculate cumulative volumes
  const bidValues: number[] = [];
  let totalBidVol = 0;
  for (const bid of bidData) {
    totalBidVol += bid.volume;
    bidValues.push(totalBidVol);
  }

  const askValues: number[] = [];
  let totalAskVol = 0;
  for (const ask of askData) {
    totalAskVol += ask.volume;
    askValues.push(totalAskVol);
  }

  // Create bid series (green)
  const bidSeries = new FastMountainRenderableSeries(wasmContext, {
    dataSeries: new XyDataSeries(wasmContext, {
      xValues: bidData.map((bid) => bid.price),
      yValues: bidValues,
    }),
    stroke: appTheme.VividGreen,
    fill: "00ff0033",
    strokeThickness: 2,
    isDigitalLine: true,
  });

  // Create ask series (red)
  const askSeries = new FastMountainRenderableSeries(wasmContext, {
    dataSeries: new XyDataSeries(wasmContext, {
      xValues: askData.map((ask) => ask.price),
      yValues: askValues,
    }),
    stroke: appTheme.VividRed,
    fill: "ff000033",
    strokeThickness: 2,
    isDigitalLine: true,
  });

  sciChartSurface.renderableSeries.add(bidSeries, askSeries);

  return { sciChartSurface };
};

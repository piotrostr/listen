import {
  CandlestickSeries,
  ColorType,
  createChart,
  HistogramSeries,
  UTCTimestamp,
} from "lightweight-charts";
import { useEffect, useRef } from "react";
import { z } from "zod";

interface ChartProps {
  mint: string;
  interval?: "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "1d";
}

const Candlestick = z.object({
  timestamp: z.number(),
  open: z.number(),
  high: z.number(),
  low: z.number(),
  close: z.number(),
  volume: z.number(),
});

type Candlestick = z.infer<typeof Candlestick>;

const CandlestickData = z.array(Candlestick);

type CandlestickData = z.infer<typeof CandlestickData>;

// Add TradingView color constants
const TV_COLORS = {
  GREEN: "#26a69a",
  RED: "#ef5350",
  GREEN_TRANSPARENT: "rgba(38, 166, 154, 0.3)",
  RED_TRANSPARENT: "rgba(239, 83, 80, 0.3)",
} as const;

export function Chart({ mint, interval = "1m" }: ChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<ReturnType<typeof createChart>>(null);
  const isDisposed = useRef(false);

  useEffect(() => {
    if (!chartContainerRef.current) return;
    isDisposed.current = false;

    // Initialize chart with container dimensions
    const container = chartContainerRef.current;
    const chart = createChart(container, {
      width: container.clientWidth,
      height: container.clientHeight,
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#d1d5db",
      },
      grid: {
        vertLines: { color: "#374151" },
        horzLines: { color: "#374151" },
      },
      timeScale: {
        timeVisible: true,
        secondsVisible: false,
        tickMarkFormatter: (time: UTCTimestamp) => {
          const date = new Date(time * 1000);
          const hours = date.getHours().toString().padStart(2, "0");
          const minutes = date.getMinutes().toString().padStart(2, "0");
          return `${hours}:${minutes}`;
        },
      },
    });

    // Create candlestick series with default settings
    const candlestickSeries = chart.addSeries(CandlestickSeries, {
      priceFormat: {
        minMove: 0.000001,
        precision: 6,
      },
      upColor: TV_COLORS.GREEN,
      downColor: TV_COLORS.RED,
      wickUpColor: TV_COLORS.GREEN,
      wickDownColor: TV_COLORS.RED,
      borderVisible: false,
    });

    candlestickSeries.priceScale().applyOptions({
      scaleMargins: {
        top: 0.1, // highest point of the series will be 10% away from the top
        bottom: 0.4, // lowest point will be 40% away from the bottom
      },
    });

    const volumeSeries = chart.addSeries(HistogramSeries, {
      priceScaleId: "", // set as an overlay
      priceFormat: {
        type: "volume",
      },
    });

    volumeSeries.priceScale().applyOptions({
      scaleMargins: {
        top: 0.7,
        bottom: 0,
      },
    });

    const fetchData = async () => {
      if (isDisposed.current) return;

      try {
        const response = await fetch(
          `https://api.listen-rs.com/v1/adapter/candlesticks?mint=${mint}&interval=${interval}`
        );
        const data = CandlestickData.parse(await response.json());

        if (isDisposed.current) return;

        // Sort in ascending order (oldest to newest)
        const sortedData = data.sort((a, b) => a.timestamp - b.timestamp);

        // Filter out outliers TODO
        const filteredData = sortedData;

        const candleData = filteredData.map((d) => ({
          time: d.timestamp as UTCTimestamp,
          open: d.open,
          high: d.high,
          low: d.low,
          close: d.close,
        }));

        console.log(candleData);

        const volumeData = filteredData.map((d) => ({
          time: d.timestamp as UTCTimestamp,
          value: d.volume,
          color:
            d.close >= d.open
              ? TV_COLORS.GREEN_TRANSPARENT
              : TV_COLORS.RED_TRANSPARENT,
        }));

        if (candleData.length > 0 && !isDisposed.current) {
          candlestickSeries.setData(candleData);
          volumeSeries.setData(volumeData);

          // Set a wider visible range
          const timeScale = chart.timeScale();
          timeScale.applyOptions({
            rightOffset: 12, // Add some space on the right
            barSpacing: 6, // Make bars a bit closer together
          });

          timeScale.fitContent();

          // Add padding on both sides by showing a subset of the total range
          const totalBars = candleData.length;
          timeScale.setVisibleLogicalRange({
            from: -5, // Show 5 bars worth of space on the left
            to: totalBars + 5, // Show 5 bars worth of space on the right (in addition to rightOffset)
          });
        }
      } catch (error) {
        if (!isDisposed.current) {
          console.error("Failed to fetch chart data:", error);
        }
      }
    };

    fetchData();
    // @ts-ignore
    chartRef.current = chart;

    const handleResize = () => {
      if (chartContainerRef.current && !isDisposed.current) {
        chart.applyOptions({
          width: chartContainerRef.current.clientWidth,
          height: chartContainerRef.current.clientHeight,
        });
      }
    };

    window.addEventListener("resize", handleResize);

    // Cleanup
    return () => {
      isDisposed.current = true;
      window.removeEventListener("resize", handleResize);
      chart.remove();
    };
  }, [mint, interval]);

  return (
    <div
      ref={chartContainerRef}
      className="w-full h-full"
      style={{ minHeight: "100%" }}
    />
  );
}

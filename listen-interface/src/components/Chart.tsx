import {
  CandlestickSeries,
  ColorType,
  createChart,
  HistogramSeries,
  UTCTimestamp,
} from "lightweight-charts";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { CandlestickData, CandlestickDataSchema } from "../hooks/types";
import { useListenMetadata } from "../hooks/useListenMetadata";

// Props for the inner chart component that receives data directly
interface InnerChartProps {
  data: CandlestickData;
}

// Props for the outer chart component that can either fetch or receive data
export interface ChartProps {
  mint: string;
  interval?: "1m" | "5m" | "15m" | "30m" | "1h" | "4h" | "1d" | "30s";
  name?: string;
  symbol?: string;
  pubkey?: string;
}

// Available time intervals for the chart
const INTERVALS = ["30s", "1m", "5m", "15m", "30m", "1h", "4h", "1d"] as const;

// Add TradingView color constants
const TV_COLORS = {
  GREEN: "#26a69a",
  RED: "#ef5350",
  GREEN_TRANSPARENT: "rgba(38, 166, 154, 0.3)",
  RED_TRANSPARENT: "rgba(239, 83, 80, 0.3)",
} as const;

export function InnerChart({ data }: InnerChartProps) {
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

    // Sort and process the data
    const sortedData = data.sort((a, b) => a.timestamp - b.timestamp);
    const filteredData = sortedData;

    const candleData = filteredData.map((d) => ({
      time: d.timestamp as UTCTimestamp,
      open: d.open,
      high: d.high,
      low: d.low,
      close: d.close,
    }));

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
  }, [data]);

  return (
    <div
      ref={chartContainerRef}
      className="w-full h-full"
      style={{ minHeight: "100%" }}
    />
  );
}

export function Chart({ mint, interval: defaultInterval = "30s" }: ChartProps) {
  // State to track the currently selected interval
  const [selectedInterval, setSelectedInterval] =
    useState<(typeof INTERVALS)[number]>(defaultInterval);
  const [isLoading, setIsLoading] = useState(true);
  const isDisposed = useRef(false);

  // Cache for storing data for different intervals
  const [dataCache, setDataCache] = useState<Record<string, CandlestickData>>(
    {}
  );

  const { data: metadata } = useListenMetadata(mint);

  // Calculate percentage change from the data
  const percentChange = useMemo(() => {
    if (
      !dataCache[selectedInterval] ||
      dataCache[selectedInterval].length < 2
    ) {
      return null;
    }

    const sortedData = [...dataCache[selectedInterval]].sort(
      (a, b) => a.timestamp - b.timestamp
    );
    const firstCandle = sortedData[0];
    const lastCandle = sortedData[sortedData.length - 1];

    const change =
      ((lastCandle.close - firstCandle.open) / firstCandle.open) * 100;
    return change;
  }, [dataCache, selectedInterval]);

  // Fetch data when mint or selected interval changes
  useEffect(() => {
    isDisposed.current = false;
    setIsLoading(true);

    const fetchData = async () => {
      if (isDisposed.current) return;

      // Check if we already have cached data for this interval
      if (dataCache[selectedInterval]) {
        setIsLoading(false);
        return;
      }

      try {
        const response = await fetch(
          `https://api.listen-rs.com/v1/adapter/candlesticks?mint=${mint}&interval=${selectedInterval}`
        );
        const responseData = CandlestickDataSchema.parse(await response.json());

        if (!isDisposed.current) {
          // Update the cache with the new data
          setDataCache((prevCache) => ({
            ...prevCache,
            [selectedInterval]: responseData,
          }));
          setIsLoading(false);
        }
      } catch (error) {
        if (!isDisposed.current) {
          console.error("Failed to fetch chart data:", error);
          setIsLoading(false);
        }
      }
    };

    fetchData();

    return () => {
      isDisposed.current = true;
    };
  }, [mint, selectedInterval]);

  // Handle interval change
  const handleIntervalChange = useCallback(
    (interval: (typeof INTERVALS)[number]) => {
      setSelectedInterval(interval);
    },
    []
  );

  // Format pubkey for display
  const formattedPubkey = useMemo(() => {
    if (!metadata?.mint) return "";
    return metadata.mint.length > 12
      ? `${metadata.mint.slice(0, 6)}...${metadata.mint.slice(-6)}`
      : metadata.mint;
  }, [metadata]);

  return (
    <div className="flex flex-col w-full h-full">
      {/* Token information and interval selection in a single row */}
      <div className="flex items-center justify-between mb-2 p-2 bg-gray-800 rounded">
        <div className="flex items-center space-x-2">
          {metadata?.mpl.symbol && (
            <span className="font-bold text-white">{metadata.mpl.symbol}</span>
          )}
          {metadata?.mpl.name && (
            <span className="text-gray-300 ml-2">{metadata.mpl.name}</span>
          )}
          {metadata?.mint && (
            <span className="text-xs text-gray-400 ml-2" title={metadata.mint}>
              ({formattedPubkey})
            </span>
          )}

          {/* Percentage change indicator */}
          {percentChange !== null && (
            <span
              className={`ml-3 font-medium ${
                percentChange >= 0 ? "text-green-400" : "text-red-400"
              }`}
            >
              {percentChange >= 0 ? "+" : ""}
              {percentChange.toFixed(2)}%
            </span>
          )}
        </div>

        {/* Interval selection buttons moved to the right */}
        <div className="flex space-x-1">
          {INTERVALS.map((interval) => (
            <button
              key={interval}
              onClick={() => handleIntervalChange(interval)}
              className={`px-2 py-1 text-xs rounded ${
                selectedInterval === interval
                  ? "bg-blue-600 text-white"
                  : "bg-gray-700 text-gray-300 hover:bg-gray-600"
              }`}
            >
              {interval}
            </button>
          ))}
        </div>
      </div>

      {/* Chart */}
      <div className="flex-grow">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            Loading...
          </div>
        ) : (
          dataCache[selectedInterval] && (
            <InnerChart data={dataCache[selectedInterval]} />
          )
        )}
      </div>
    </div>
  );
}

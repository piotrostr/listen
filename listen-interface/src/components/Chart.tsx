import {
  CandlestickSeries,
  ColorType,
  createChart,
  CrosshairMode,
  HistogramSeries,
  ISeriesApi,
  UTCTimestamp,
} from "lightweight-charts";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { CandlestickData, CandlestickDataSchema } from "../hooks/types";
import { useListenMetadata } from "../hooks/useListenMetadata";
import { useTokenStore } from "../store/tokenStore";
import { Socials } from "./Socials";

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
  const candlestickSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeSeriesRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const isDisposed = useRef(false);
  const lastDataRef = useRef<CandlestickData>([]);

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
      crosshair: {
        mode: CrosshairMode.Normal,
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

    candlestickSeriesRef.current = candlestickSeries;

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

    volumeSeriesRef.current = volumeSeries;

    volumeSeries.priceScale().applyOptions({
      scaleMargins: {
        top: 0.7,
        bottom: 0,
      },
    });

    // Save the initial data
    lastDataRef.current = data;

    // Initial data loading - important to use the correct timestamp format
    if (data.length > 0) {
      const formattedData = data.map((d) => ({
        time: d.timestamp as UTCTimestamp,
        open: d.open,
        high: d.high,
        low: d.low,
        close: d.close,
      }));

      const volumeData = data.map((d) => ({
        time: d.timestamp as UTCTimestamp,
        value: d.volume,
        color:
          d.close >= d.open
            ? TV_COLORS.GREEN_TRANSPARENT
            : TV_COLORS.RED_TRANSPARENT,
      }));

      candlestickSeries.setData(formattedData);
      volumeSeries.setData(volumeData);

      // Set a wider visible range
      const timeScale = chart.timeScale();
      timeScale.applyOptions({
        rightOffset: 12, // Add some space on the right
        barSpacing: 6, // Make bars a bit closer together
      });

      timeScale.fitContent();

      // Add padding on both sides by showing a subset of the total range
      const totalBars = formattedData.length;
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
  }, []); // Only on mount

  useEffect(() => {
    if (
      isDisposed.current ||
      !data.length ||
      !candlestickSeriesRef.current ||
      !volumeSeriesRef.current
    )
      return;

    // Just get the newest candle to update
    const sortedData = [...data].sort((a, b) => b.timestamp - a.timestamp);
    const newestCandle = sortedData[0];

    // Update with the latest candle
    candlestickSeriesRef.current.update({
      time: newestCandle.timestamp as UTCTimestamp,
      open: newestCandle.open,
      high: newestCandle.high,
      low: newestCandle.low,
      close: newestCandle.close,
    });

    volumeSeriesRef.current.update({
      time: newestCandle.timestamp as UTCTimestamp,
      value: newestCandle.volume,
      color:
        newestCandle.close >= newestCandle.open
          ? TV_COLORS.GREEN_TRANSPARENT
          : TV_COLORS.RED_TRANSPARENT,
    });
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
  const [data, setData] = useState<CandlestickData>([]);
  const isDisposed = useRef(false);

  // Subscribe to token store updates
  const latestUpdate = useTokenStore((state) => state.latestUpdate);

  const { data: metadata } = useListenMetadata(mint);

  const percentChange = useMemo(() => {
    if (!data || data.length < 2) {
      return null;
    }

    const sortedData = [...data].sort((a, b) => a.timestamp - b.timestamp);
    const firstCandle = sortedData[0];
    const lastCandle = sortedData[sortedData.length - 1];

    const change =
      ((lastCandle.close - firstCandle.open) / firstCandle.open) * 100;
    return change;
  }, [data]);

  useEffect(() => {
    if (!latestUpdate || isDisposed.current || latestUpdate.pubkey !== mint) {
      return;
    }

    setData((prevData) => {
      if (!prevData.length) return prevData;

      const newData = [...prevData];
      const lastCandle = newData[newData.length - 1];
      const currentTimestamp = Math.floor(Date.now() / 1000); // Current timestamp in seconds

      // Simple function to check if we need a new candle based on interval
      const needsNewCandle = () => {
        const lastCandleTime = new Date(lastCandle.timestamp * 1000);
        const currentTime = new Date(currentTimestamp * 1000);

        switch (selectedInterval) {
          case "30s":
            return (
              Math.floor(lastCandleTime.getTime() / 30000) !==
              Math.floor(currentTime.getTime() / 30000)
            );
          case "1m":
            return (
              lastCandleTime.getUTCMinutes() !== currentTime.getUTCMinutes() ||
              lastCandleTime.getUTCHours() !== currentTime.getUTCHours() ||
              lastCandleTime.getUTCDate() !== currentTime.getUTCDate()
            );
          case "5m":
            return (
              Math.floor(lastCandleTime.getUTCMinutes() / 5) !==
                Math.floor(currentTime.getUTCMinutes() / 5) ||
              lastCandleTime.getUTCHours() !== currentTime.getUTCHours() ||
              lastCandleTime.getUTCDate() !== currentTime.getUTCDate()
            );
          case "15m":
            return (
              Math.floor(lastCandleTime.getUTCMinutes() / 15) !==
                Math.floor(currentTime.getUTCMinutes() / 15) ||
              lastCandleTime.getUTCHours() !== currentTime.getUTCHours() ||
              lastCandleTime.getUTCDate() !== currentTime.getUTCDate()
            );
          case "30m":
            return (
              Math.floor(lastCandleTime.getUTCMinutes() / 30) !==
                Math.floor(currentTime.getUTCMinutes() / 30) ||
              lastCandleTime.getUTCHours() !== currentTime.getUTCHours() ||
              lastCandleTime.getUTCDate() !== currentTime.getUTCDate()
            );
          case "1h":
            return (
              lastCandleTime.getUTCHours() !== currentTime.getUTCHours() ||
              lastCandleTime.getUTCDate() !== currentTime.getUTCDate()
            );
          case "4h":
            return (
              Math.floor(lastCandleTime.getUTCHours() / 4) !==
                Math.floor(currentTime.getUTCHours() / 4) ||
              lastCandleTime.getUTCDate() !== currentTime.getUTCDate()
            );
          case "1d":
            return lastCandleTime.getUTCDate() !== currentTime.getUTCDate();
          default:
            return false;
        }
      };

      // Check if we need a new candle
      if (needsNewCandle()) {
        // Create a new candle
        const newCandle = {
          timestamp: currentTimestamp,
          open: latestUpdate.price,
          high: latestUpdate.price,
          low: latestUpdate.price,
          close: latestUpdate.price,
          volume: latestUpdate.swap_amount,
        };
        return [...newData, newCandle];
      } else {
        // Update the existing candle
        const updatedCandle = {
          ...lastCandle,
          close: latestUpdate.price,
          high: Math.max(lastCandle.high, latestUpdate.price),
          low: Math.min(lastCandle.low, latestUpdate.price),
          volume: lastCandle.volume + latestUpdate.swap_amount,
        };

        newData[newData.length - 1] = updatedCandle;
        return newData;
      }
    });
  }, [latestUpdate, mint, selectedInterval]);

  // Fetch data when mint or selected interval changes
  useEffect(() => {
    isDisposed.current = false;
    setIsLoading(true);

    const fetchData = async () => {
      if (isDisposed.current) return;

      try {
        const response = await fetch(
          `https://api.listen-rs.com/v1/adapter/candlesticks?mint=${mint}&interval=${selectedInterval}`
        );
        const responseData = CandlestickDataSchema.parse(await response.json());

        if (!isDisposed.current) {
          setData(responseData);
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

  // Handle select change for mobile dropdown
  const handleSelectChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      const newInterval = e.target.value as (typeof INTERVALS)[number];
      setSelectedInterval(newInterval);
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
      <div className="flex items-center justify-between mb-2 p-3 backdrop-blur-sm">
        <div className="flex items-center">
          {/* Add token image with proper spacing */}
          {metadata?.mpl.ipfs_metadata?.image && (
            <div className="w-8 h-8 relative rounded-full overflow-hidden mr-3">
              <img
                src={metadata.mpl.ipfs_metadata.image.replace(
                  "cf-ipfs.com",
                  "ipfs.io"
                )}
                alt={metadata?.mpl.symbol || "Token"}
                className="w-full h-full object-cover"
              />
            </div>
          )}
          <div className="flex flex-col">
            <div className="flex items-center space-x-2">
              {metadata?.mpl.symbol && (
                <span className="font-bold text-white">
                  {metadata.mpl.symbol}
                </span>
              )}
              {metadata?.mpl.name && (
                <span className="text-white ml-2 hidden lg:block">
                  {metadata.mpl.name}
                </span>
              )}
              {metadata?.mint && (
                <span
                  className="text-xs text-white/70 ml-2 hidden lg:block"
                  title={metadata.mint}
                >
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
            <Socials tokenMetadata={metadata ?? null} pubkey={mint} />
          </div>
        </div>

        {/* Interval selection moved to the right */}
        <div className="flex space-x-1 ml-auto">
          {/* Mobile dropdown */}
          <div className="md:hidden">
            <select
              value={selectedInterval}
              onChange={handleSelectChange}
              className="bg-black/40 text-white rounded px-2 py-1 text-xs border-none focus:ring-2 focus:ring-[#2D2D2D]"
            >
              {INTERVALS.map((interval) => (
                <option key={interval} value={interval}>
                  {interval}
                </option>
              ))}
            </select>
          </div>

          {/* Desktop buttons */}
          <div className="hidden md:flex space-x-1">
            {INTERVALS.map((interval) => (
              <button
                key={interval}
                onClick={() => handleIntervalChange(interval)}
                className={`px-2 py-1 text-xs rounded ${
                  selectedInterval === interval
                    ? "bg-[#2D2D2D] text-white hover:bg-[#2D2D2D]"
                    : "bg-transparent text-white hover:bg-[#2D2D2D]"
                }`}
              >
                {interval}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Chart */}
      <div className="flex-grow">
        {isLoading ? (
          <div className="flex items-center justify-center h-full text-white">
            Loading...
          </div>
        ) : (
          data && <InnerChart data={data} />
        )}
      </div>
    </div>
  );
}

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
import { z } from "zod";
import { useListenMetadata } from "../hooks/useListenMetadata";
import { CandlestickData, CandlestickDataSchema } from "../lib/types";
import { useTokenStore } from "../store/tokenStore";
import { Socials } from "./Socials";

// Props for the inner chart component that receives data directly
interface InnerChartProps {
  data: CandlestickData;
  displayOhlc?: boolean;
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

// GeckoTerminal API schemas
const GeckoTerminalPoolSchema = z.object({
  data: z.array(
    z.object({
      attributes: z.object({
        address: z.string(),
        volume_usd: z.object({
          h24: z.string(),
        }),
      }),
    })
  ),
});

const GeckoTerminalOHLCVSchema = z.object({
  data: z.object({
    attributes: z.object({
      ohlcv_list: z.array(
        z.array(z.union([z.number(), z.string()]))
      ),
    }),
  }),
});

// Cache for pool addresses to prevent repeated API calls
const poolAddressCache: Record<string, { address: string; timestamp: number }> = {};
const CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

// Add TradingView color constants
const TV_COLORS = {
  GREEN: "#26a69a",
  RED: "#ef5350",
  GREEN_TRANSPARENT: "rgba(38, 166, 154, 0.3)",
  RED_TRANSPARENT: "rgba(239, 83, 80, 0.3)",
} as const;

// Find the most liquid pool for a token on Solana
async function findSolanaPoolAddress(tokenAddress: string): Promise<string | null> {
  const cacheKey = `solana:${tokenAddress}`;
  const cached = poolAddressCache[cacheKey];
  
  if (cached && Date.now() - cached.timestamp < CACHE_DURATION) {
    return cached.address;
  }

  try {
    const response = await fetch(
      `https://api.geckoterminal.com/api/v2/networks/solana/tokens/${tokenAddress}/pools`,
      {
        headers: {
          Accept: "application/json;version=20230302",
        },
      }
    );

    if (!response.ok) {
      console.error(`Failed to fetch pools for token ${tokenAddress}`);
      return null;
    }

    const json = await response.json();
    const result = GeckoTerminalPoolSchema.safeParse(json);

    if (!result.success) {
      console.error("Failed to parse pool response:", result.error);
      return null;
    }

    // Sort pools by 24h volume to get the most active pool
    const sortedPools = result.data.data.sort(
      (a, b) =>
        parseFloat(b.attributes.volume_usd.h24) -
        parseFloat(a.attributes.volume_usd.h24)
    );

    if (sortedPools.length > 0) {
      const address = sortedPools[0].attributes.address;
      poolAddressCache[cacheKey] = {
        address,
        timestamp: Date.now(),
      };
      return address;
    }

    return null;
  } catch (error) {
    console.error("Failed to fetch pool address:", error);
    return null;
  }
}

// Map chart intervals to GeckoTerminal timeframes
function mapIntervalToGeckoTimeframe(interval: string): { timeframe: string; aggregate: number } {
  switch (interval) {
    case "30s":
    case "1m":
      return { timeframe: "minute", aggregate: 1 };
    case "5m":
      return { timeframe: "minute", aggregate: 5 };
    case "15m":
      return { timeframe: "minute", aggregate: 15 };
    case "30m":
      return { timeframe: "hour", aggregate: 1 }; // Use 1h as closest available
    case "1h":
      return { timeframe: "hour", aggregate: 1 };
    case "4h":
      return { timeframe: "hour", aggregate: 4 };
    case "1d":
      return { timeframe: "day", aggregate: 1 };
    default:
      return { timeframe: "minute", aggregate: 1 };
  }
}

// Fetch OHLC data from GeckoTerminal
async function fetchGeckoTerminalOHLC(
  poolAddress: string,
  interval: string
): Promise<CandlestickData | null> {
  try {
    const { timeframe, aggregate } = mapIntervalToGeckoTimeframe(interval);
    
    const response = await fetch(
      `https://api.geckoterminal.com/api/v2/networks/solana/pools/${poolAddress}/ohlcv/${timeframe}?aggregate=${aggregate}&limit=1000`,
      {
        headers: {
          Accept: "application/json;version=20230302",
        },
      }
    );

    if (!response.ok) {
      console.error(`Failed to fetch OHLC data for pool ${poolAddress}`);
      return null;
    }

    const json = await response.json();
    const result = GeckoTerminalOHLCVSchema.safeParse(json);

    if (!result.success) {
      console.error("Failed to parse OHLC response:", result.error);
      return null;
    }

    // Convert GeckoTerminal OHLCV format to our Candlestick format
    const candlesticks: CandlestickData = result.data.data.attributes.ohlcv_list.map((item) => {
      const [timestamp, open, high, low, close, volume] = item;
      return {
        timestamp: typeof timestamp === "number" ? timestamp : parseInt(timestamp as string),
        open: typeof open === "number" ? open : parseFloat(open as string),
        high: typeof high === "number" ? high : parseFloat(high as string),
        low: typeof low === "number" ? low : parseFloat(low as string),
        close: typeof close === "number" ? close : parseFloat(close as string),
        volume: typeof volume === "number" ? volume : parseFloat(volume as string),
      };
    });

    return candlesticks;
  } catch (error) {
    console.error("Failed to fetch GeckoTerminal OHLC data:", error);
    return null;
  }
}

export function InnerChart({ data, displayOhlc }: InnerChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<ReturnType<typeof createChart>>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
  const volumeSeriesRef = useRef<ISeriesApi<"Histogram"> | null>(null);
  const isDisposed = useRef(false);
  const lastDataRef = useRef<CandlestickData>([]);
  // Add state for crosshair data
  const [crosshairData, setCrosshairData] = useState<{
    open?: number;
    high?: number;
    low?: number;
    close?: number;
    volume?: number;
    time?: string;
  } | null>(null);

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

    // Subscribe to crosshair movement
    if (displayOhlc) {
      chart.subscribeCrosshairMove((param) => {
        if (
          isDisposed.current ||
          !param.time ||
          !candlestickSeriesRef.current ||
          !volumeSeriesRef.current
        )
          return;

        const candleData = param.seriesData.get(
          candlestickSeriesRef.current
        ) as
          | {
              open: number;
              high: number;
              low: number;
              close: number;
              time: UTCTimestamp;
            }
          | undefined;

        const volumeData = param.seriesData.get(volumeSeriesRef.current) as
          | {
              value: number;
              time: UTCTimestamp;
            }
          | undefined;

        if (candleData) {
          const date = new Date(candleData.time * 1000);
          const formattedTime = date.toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
            hour12: false,
          });

          setCrosshairData({
            open: candleData.open,
            high: candleData.high,
            low: candleData.low,
            close: candleData.close,
            volume: volumeData?.value,
            time: formattedTime,
          });
        } else {
          setCrosshairData(null);
        }
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

  const formatNumber = (num: number | undefined): string => {
    if (num === undefined) return "-";

    if (num >= 1000000000) {
      return (num / 1000000000).toFixed(2) + "B";
    } else if (num >= 1000000) {
      return (num / 1000000).toFixed(2) + "M";
    } else if (num >= 1000) {
      return (num / 1000).toFixed(2) + "K";
    } else {
      return num.toFixed(2);
    }
  };

  return (
    <div className="w-full h-full relative">
      <div
        ref={chartContainerRef}
        className="w-full h-full"
        style={{ minHeight: "100%" }}
      />

      {/* Updated OHLC Display - single line format */}
      {crosshairData && displayOhlc && (
        <div className="absolute top-3 left-3 bg-[#121621] py-1 px-2 rounded text-xs font-mono z-10">
          <div className="flex space-x-3">
            <span>O {crosshairData.open}</span>
            <span>H {crosshairData.high}</span>
            <span>L {crosshairData.low}</span>
            <span>C {crosshairData.close}</span>
            <span className="text-[#3CD2B9]">
              {formatNumber(crosshairData.volume)}
            </span>
          </div>
        </div>
      )}
    </div>
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
        // First try to get data from GeckoTerminal
        const poolAddress = await findSolanaPoolAddress(mint);
        let responseData: CandlestickData | null = null;

        if (poolAddress) {
          console.log(`Found pool address ${poolAddress} for token ${mint}, fetching from GeckoTerminal`);
          responseData = await fetchGeckoTerminalOHLC(poolAddress, selectedInterval);
        }

        // If GeckoTerminal fails or no pool found, fall back to custom API
        if (!responseData || responseData.length === 0) {
          console.log(`Falling back to custom API for token ${mint}`);
          const response = await fetch(
            // use prod for charts always
            `https://api.listen-rs.com/v1/adapter/candlesticks?mint=${mint}&interval=${selectedInterval}`
          );
          responseData = CandlestickDataSchema.parse(await response.json());
        }

        if (!isDisposed.current) {
          setData(responseData || []);
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

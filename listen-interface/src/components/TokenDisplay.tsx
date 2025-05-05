import { Token } from "../lib/types";

const PercentageChange = ({ pct_change }: { pct_change: number }) => {
  const isPositive = pct_change >= 0;
  const formattedChange = `${isPositive ? "+" : ""}${pct_change.toFixed(2)}%`;

  return (
    <div
      className={`
        flex justify-center items-center p-2 rounded-full w-14 h-7 font-dm-sans
        ${isPositive ? "bg-pump-green-bg" : "bg-pump-red-bg"}
      `}
    >
      <span
        className={`
          text-xs font-normal leading-3
          ${isPositive ? "text-pump-green" : "text-pump-red"}
        `}
      >
        {formattedChange}
      </span>
    </div>
  );
};

const Container = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="flex flex-col items-start p-0 w-full h-[205px] bg-[#0d0d0e] border-[1px] border-[#1e1e21] rounded-[20px]">
      {children}
    </div>
  );
};

const TokenImage = ({ src, alt }: { src: string; alt: string }) => {
  return (
    <img
      src={src}
      alt={alt}
      className="w-[56px] h-[56px] border-[1px] border-[#404040] rounded-full"
    />
  );
};

const ChartLine = ({
  ema_price_ticks,
  pct_change,
}: {
  ema_price_ticks: { price: number }[];
  pct_change: number;
}) => {
  if (!ema_price_ticks?.length) return null;

  const isPositive = pct_change >= 0;
  const lineColor = isPositive ? "#8DFC63" : "#8A5EFB";
  const gradientStartColor = isPositive ? "#8DFC63" : "#8057FB";
  const gradientEndColor = isPositive ? "#8DFC63" : "#F72777";

  // Scale points to view
  const prices = ema_price_ticks.map((tick) => tick.price);
  const minPrice = Math.min(...prices);
  const maxPrice = Math.max(...prices);
  const priceRange = maxPrice - minPrice;

  // Increase padding to 10% at bottom
  const paddedMinPrice = minPrice - priceRange * 0.1;
  const paddedPriceRange = priceRange * 1.2; // Increase range by 20% for padding

  const points = ema_price_ticks.map((tick, i) => {
    const x = (i / (ema_price_ticks.length - 1)) * 358;
    const y = 105 - ((tick.price - paddedMinPrice) / paddedPriceRange) * 98;
    return [x, y];
  });

  // Create smooth curve using quadratic Bézier
  const linePath = points.reduce((path, point, i, points) => {
    if (i === 0) return `M ${point[0]},${point[1]}`;
    if (i === points.length - 1) return `${path} L ${point[0]},${point[1]}`;

    const next = points[i + 1];
    const controlX = (point[0] + next[0]) / 2;
    const controlY = (point[1] + next[1]) / 2;

    return `${path} Q ${point[0]},${point[1]} ${controlX},${controlY}`;
  }, "");

  const fillPath = `${linePath} L358,105 L0,105 Z`;

  return (
    <div className="w-full h-[100px]">
      <svg
        width="100%"
        height="100%"
        preserveAspectRatio="none"
        viewBox="0 0 358 105"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient
            id="chartGradient"
            x1="179"
            y1="5"
            x2="179"
            y2="105"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor={gradientStartColor} />
            <stop offset="1" stopColor={gradientEndColor} stopOpacity="0" />
          </linearGradient>
        </defs>

        <path d={fillPath} fill="url(#chartGradient)" fillOpacity="0.16" />

        <path
          d={linePath}
          stroke={lineColor}
          strokeWidth="4"
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        />
      </svg>
    </div>
  );
};

export function TokenDisplay({ token }: { token: Token }) {
  const { metadata, price_info } = token;
  const name =
    metadata?.mpl?.name.length > 15 ? metadata.mpl.symbol : metadata.mpl.name;
  return (
    <Container>
      <div className="flex flex-row p-4 items-center">
        <TokenImage
          src={metadata?.mpl?.ipfs_metadata?.image}
          alt={metadata?.mpl?.name}
        />
        <div className="flex flex-col p-2">
          <div className="flex flex-row items-center space-x-2">
            <div className="font-space-grotesk font-normal text-2xl leading-8 tracking-[-0.03em] text-center align-middle">
              {metadata?.mpl?.name}
            </div>
            {price_info?.pct_change && (
              <PercentageChange pct_change={price_info?.pct_change} />
            )}
          </div>
          <div className="font-dm-sans font-light text-[14px] leading-[16px] tracking-[0%] align-middle text-[#868686]">
            {price_info?.latest_price.toFixed(8)}
          </div>
        </div>
      </div>
      {price_info?.ema_price_ticks && (
        <ChartLine
          ema_price_ticks={price_info.ema_price_ticks}
          pct_change={price_info.pct_change || 0}
        />
      )}
    </Container>
  );
}

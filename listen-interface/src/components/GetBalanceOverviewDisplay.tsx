import { HyperliquidPortfolioOverview } from "../lib/hype-types";

const Container = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="flex flex-col items-start p-0 w-full bg-[#0d0d0e] border-[1px] border-[#1e1e21] rounded-[20px] mt-2">
      {children}
    </div>
  );
};

const SectionHeader = ({ title }: { title: string }) => {
  return (
    <div className="flex flex-col w-full border-b border-[#1e1e21] pb-2 mb-2">
      <div className="flex flex-row justify-between items-center">
        <div className="font-space-grotesk font-normal text-base leading-6 tracking-[-0.03em] text-white">
          {title}
        </div>
      </div>
    </div>
  );
};

const formatLiquidationPrice = (price: string) => {
  const numPrice = parseFloat(price);
  if (numPrice > 999) {
    return numPrice.toFixed(2);
  }
  return numPrice.toFixed(6);
};

const PositionRow = ({
  coin,
  amount,
  type,
  entryPx,
  pnl,
  leverage,
  liquidationPx,
  returnOnEquity,
  positionValue,
  showLongShort = true,
}: {
  coin: string;
  amount: string;
  type: string;
  entryPx?: string;
  pnl?: string;
  leverage?: number;
  liquidationPx?: string;
  returnOnEquity?: string;
  positionValue?: string;
  showLongShort?: boolean;
}) => {
  const positionSize = parseFloat(amount);
  const isLong = positionSize > 0;
  const isShort = positionSize < 0;

  return (
    <div className="flex flex-col w-full py-1 border-b border-[#1e1e21] last:border-b-0">
      <div className="flex flex-row justify-between items-center">
        <div className="flex flex-col">
          <div className="flex flex-row items-center gap-2">
            <div className="font-dm-sans font-normal text-sm text-white">
              {coin}
            </div>
            {leverage && (
              <div className="font-dm-sans font-normal text-xs text-[#868686]">
                {leverage}x
              </div>
            )}
            {showLongShort && (isLong || isShort) && (
              <div
                className={`font-dm-sans font-normal text-xs px-1 rounded ${isLong ? "text-pump-green" : "text-pump-red"}`}
              >
                {isLong ? "Long" : "Short"}
              </div>
            )}
          </div>
          <div className="font-dm-sans font-light text-[10px] text-[#868686]">
            {type}
          </div>
        </div>
        <div className="flex flex-col items-end">
          <div className="font-dm-sans font-normal text-sm text-white">
            {Math.abs(positionSize).toFixed(4)} {coin}
          </div>
          {positionValue && (
            <div className="font-dm-sans font-light text-xs text-[#868686]">
              ${parseFloat(positionValue).toFixed(2)}
            </div>
          )}
        </div>
      </div>

      {/* Additional info for perpetual positions */}
      {(entryPx || pnl || liquidationPx) && (
        <div className="flex flex-row justify-between items-center mt-1 text-xs">
          <div className="flex flex-row gap-3">
            {entryPx && (
              <span className="text-[#868686]">Entry: ${entryPx}</span>
            )}
            {liquidationPx && (
              <span className="text-pump-red">
                Liq: ${formatLiquidationPrice(liquidationPx)}
              </span>
            )}
          </div>
          {pnl && (
            <span
              className={`${parseFloat(pnl) >= 0 ? "text-pump-green" : "text-pump-red"}`}
            >
              ${parseFloat(pnl).toFixed(4)}
              {returnOnEquity && (
                <span className="ml-1">
                  ({(parseFloat(returnOnEquity) * 100).toFixed(2)}%)
                </span>
              )}
            </span>
          )}
        </div>
      )}
    </div>
  );
};

export function GetBalanceOverviewDisplay({
  balanceOverview,
}: {
  balanceOverview: HyperliquidPortfolioOverview;
}) {
  // Get all positions (spot and perpetual)
  const spotPositions = balanceOverview.spotBalances.balances.filter(
    (balance) => parseFloat(balance.total) > 0
  );

  const perpPositions = balanceOverview.perpBalances.assetPositions.filter(
    (position) => parseFloat(position.position.szi) !== 0
  );

  // Check if we have USDC available balance to show
  const accountValue = parseFloat(
    balanceOverview.perpBalances.marginSummary.accountValue
  );

  return (
    <Container>
      <div className="flex flex-col p-4 w-full">
        <SectionHeader title="Hyperliquid Balances" />

        <div className="flex flex-col gap-1">
          {/* USDC Available Balance (show as perpetual) */}
          {accountValue > 0 && (
            <PositionRow
              key="usdc-perpetual"
              coin="USDC"
              amount={accountValue.toString()}
              type={`Perpetual`}
              showLongShort={false}
            />
          )}

          {/* Perpetual positions (excluding USDC available balance) */}
          {perpPositions.map((position) => (
            <PositionRow
              key={`perp-${position.position.coin}`}
              coin={position.position.coin}
              amount={position.position.szi}
              type="Perpetual"
              entryPx={position.position.entryPx}
              pnl={position.position.unrealizedPnl}
              leverage={position.position.leverage.value}
              liquidationPx={position.position.liquidationPx}
              returnOnEquity={position.position.returnOnEquity}
              positionValue={position.position.positionValue}
              showLongShort={true}
            />
          ))}

          {/* Spot positions */}
          {spotPositions.map((balance) => (
            <PositionRow
              key={`spot-${balance.coin}`}
              coin={balance.coin}
              amount={balance.total}
              type="Spot"
              showLongShort={false}
            />
          ))}
        </div>
      </div>
    </Container>
  );
}

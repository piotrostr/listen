import { HyperliquidPortfolioOverview } from "../lib/hype-types";

const Container = ({ children }: { children: React.ReactNode }) => {
  return (
    <div className="flex flex-col items-start p-0 w-full bg-[#0d0d0e] border-[1px] border-[#1e1e21] rounded-[20px]">
      {children}
    </div>
  );
};

const SectionHeader = ({ title, value }: { title: string; value: string }) => {
  return (
    <div className="flex flex-row justify-between items-center w-full p-4 border-b border-[#1e1e21]">
      <div className="font-space-grotesk font-normal text-lg leading-6 tracking-[-0.03em] text-white">
        {title}
      </div>
      <div className="font-dm-sans font-medium text-sm leading-4 text-[#868686]">
        {value}
      </div>
    </div>
  );
};

const BalanceItem = ({
  coin,
  balance,
  value,
  pnl,
}: {
  coin: string;
  balance: string;
  value: string;
  pnl?: string;
}) => {
  const isPositive = pnl ? parseFloat(pnl) >= 0 : true;

  return (
    <div className="flex flex-row justify-between items-center w-full p-4 border-b border-[#1e1e21] last:border-b-0">
      <div className="flex flex-col">
        <div className="font-dm-sans font-medium text-sm leading-4 text-white">
          {coin}
        </div>
        <div className="font-dm-sans font-light text-xs leading-3 text-[#868686]">
          {balance}
        </div>
      </div>
      <div className="flex flex-col items-end">
        <div className="font-dm-sans font-medium text-sm leading-4 text-white">
          ${parseFloat(value).toFixed(2)}
        </div>
        {pnl && (
          <div
            className={`font-dm-sans font-light text-xs leading-3 ${
              isPositive ? "text-pump-green" : "text-pump-red"
            }`}
          >
            {isPositive ? "+" : ""}
            {parseFloat(pnl).toFixed(2)}
          </div>
        )}
      </div>
    </div>
  );
};

const SummaryCard = ({
  title,
  accountValue,
  totalMarginUsed,
  withdrawable,
}: {
  title: string;
  accountValue: string;
  totalMarginUsed: string;
  withdrawable?: string;
}) => {
  return (
    <div className="w-full p-4 bg-[#1a1a1c] border border-[#2a2a2c] rounded-[12px] m-4">
      <div className="font-dm-sans font-medium text-sm leading-4 text-[#868686] mb-2">
        {title}
      </div>
      <div className="flex flex-col space-y-1">
        <div className="flex justify-between">
          <span className="font-dm-sans font-light text-xs text-[#868686]">
            Account Value:
          </span>
          <span className="font-dm-sans font-medium text-xs text-white">
            ${parseFloat(accountValue).toFixed(2)}
          </span>
        </div>
        <div className="flex justify-between">
          <span className="font-dm-sans font-light text-xs text-[#868686]">
            Margin Used:
          </span>
          <span className="font-dm-sans font-medium text-xs text-white">
            ${parseFloat(totalMarginUsed).toFixed(2)}
          </span>
        </div>
        {withdrawable && (
          <div className="flex justify-between">
            <span className="font-dm-sans font-light text-xs text-[#868686]">
              Withdrawable:
            </span>
            <span className="font-dm-sans font-medium text-xs text-white">
              ${parseFloat(withdrawable).toFixed(2)}
            </span>
          </div>
        )}
      </div>
    </div>
  );
};

export function GetBalanceOverviewDisplay({
  balanceOverview,
}: {
  balanceOverview: HyperliquidPortfolioOverview;
}) {
  const { spotBalances, perpBalances } = balanceOverview;

  return (
    <Container>
      <div className="w-full">
        {/* Perpetual Summary */}
        <SectionHeader
          title="Perpetual Positions"
          value={`${perpBalances.assetPositions.length} positions`}
        />

        <SummaryCard
          title="Perpetual Summary"
          accountValue={perpBalances.marginSummary.accountValue}
          totalMarginUsed={perpBalances.marginSummary.totalMarginUsed}
          withdrawable={perpBalances.withdrawable}
        />

        {/* Perpetual Positions */}
        {perpBalances.assetPositions.length > 0 ? (
          perpBalances.assetPositions.map((position, index) => (
            <BalanceItem
              key={`${position.position.coin}-${index}`}
              coin={position.position.coin}
              balance={position.position.szi}
              value={position.position.positionValue}
              pnl={position.position.unrealizedPnl}
            />
          ))
        ) : (
          <div className="p-4 text-center">
            <div className="font-dm-sans font-light text-sm text-[#868686]">
              No perpetual positions
            </div>
          </div>
        )}

        {/* Spot Summary */}
        <SectionHeader
          title="Spot Balances"
          value={`${spotBalances.balances.length} assets`}
        />

        {/* Spot Balances */}
        {spotBalances.balances.length > 0 ? (
          spotBalances.balances.map((balance, index) => (
            <BalanceItem
              key={`${balance.coin}-${index}`}
              coin={balance.coin}
              balance={balance.hold}
              value={balance.total}
            />
          ))
        ) : (
          <div className="p-4 text-center">
            <div className="font-dm-sans font-light text-sm text-[#868686]">
              No spot balances
            </div>
          </div>
        )}
      </div>
    </Container>
  );
}

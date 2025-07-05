import { z } from "zod";
import { useToken } from "../hooks/useToken";
import { formatAmount } from "../lib/util";

export const TokenBalanceSchema = z.object({
  balance: z.string(),
  decimals: z.number(),
  address: z.string(),
  chain_id: z.number(),
});
type TokenBalance = z.infer<typeof TokenBalanceSchema>;

export const TokenBalanceDisplay = ({
  tokenBalance,
}: {
  tokenBalance: TokenBalance;
}) => {
  const { balance, decimals, address, chain_id } = tokenBalance;

  // Fetch token metadata
  const { data: token } = useToken(address, chain_id.toString());

  // Calculate UI amount
  const uiAmount = formatAmount(balance, decimals);

  return (
    <div className="flex items-center gap-2 mr-4">
      {token?.logoURI ? (
        <img
          src={token.logoURI}
          alt={token.name ?? address}
          className="w-6 h-6 rounded-full shrink-0"
        />
      ) : (
        <div className="w-6 h-6 rounded-full bg-gray-700 flex items-center justify-center shrink-0">
          <span className="text-gray-400 text-xs">?</span>
        </div>
      )}
      <span className="text-sm text-gray-300">{uiAmount}</span>
    </div>
  );
};

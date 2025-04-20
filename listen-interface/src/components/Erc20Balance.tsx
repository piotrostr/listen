import { z } from "zod";
import { useToken } from "../hooks/useToken";
import { formatAmount } from "../hooks/util";

export const Erc20BalanceSchema = z.object({
  balance: z.string(),
  decimals: z.number(),
  token_address: z.string(),
  chain_id: z.number(),
});
type Erc20Balance = z.infer<typeof Erc20BalanceSchema>;

export const Erc20Balance = ({
  erc20Balance,
}: {
  erc20Balance: Erc20Balance;
}) => {
  const { balance, decimals, token_address, chain_id } = erc20Balance;

  // Fetch token metadata
  const { data: token } = useToken(token_address, chain_id.toString());

  // Calculate UI amount
  const uiAmount = formatAmount(balance, decimals);

  return (
    <div className="flex items-center gap-2 mr-4">
      {token?.logoURI ? (
        <img
          src={token.logoURI}
          alt={token.name ?? token_address}
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

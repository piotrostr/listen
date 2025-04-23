import { useSolanaWallets } from "@privy-io/react-auth";
import { Connection, PublicKey } from "@solana/web3.js";
import { useQuery } from "@tanstack/react-query";
import { decodeTokenAccount } from "./util";

const LISTEN_ADDRESS = "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump";
const connection = new Connection(import.meta.env.VITE_RPC_URL);

async function getBalance(address: string) {
  try {
    const accounts = await connection.getTokenAccountsByOwner(
      new PublicKey(address),
      { mint: new PublicKey(LISTEN_ADDRESS) }
    );
    if (!accounts.value.length) return 0;
    const account = decodeTokenAccount(accounts.value[0].account.data);
    return Number(account.amount);
  } catch {
    return 0;
  }
}

export const useListenBalance = () => {
  const { wallets } = useSolanaWallets();
  const addresses = wallets.map((w) => w.address).filter(Boolean);

  return useQuery({
    queryKey: ["listenBalance", addresses],
    queryFn: async () => {
      const balances = await Promise.all(addresses.map(getBalance));
      return balances.reduce((a, b) => a + b, 0);
    },
    enabled: addresses.length > 0,
  });
};

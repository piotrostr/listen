import { useSolanaWallets } from "@privy-io/react-auth";
import { PublicKey } from "@solana/web3.js";
import { useQuery } from "@tanstack/react-query";

export const usePrivyWallet = () => {
  const { ready, wallets } = useSolanaWallets();
  const wallet = wallets.find(
    (wallet) => wallet.type === "solana" && wallet.walletClientType === "privy",
  );
  return useQuery<PublicKey | null, Error>({
    queryKey: ["privy-wallet"],
    queryFn: async () => {
      if (!wallet) {
        return null;
      }
      return new PublicKey(wallet.address);
    },
    enabled: ready && !!wallet,
    staleTime: 20000,
    refetchInterval: 20000,
  });
};

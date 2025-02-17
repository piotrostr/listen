import { useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { useQuery, UseQueryResult } from "@tanstack/react-query";

interface WalletAddresses {
  solanaWallet: string;
  evmWallet: string;
}

export const usePrivyWallets = (): UseQueryResult<
  WalletAddresses | null,
  Error
> => {
  const { ready: solanaReady, wallets: solanaWallets } = useSolanaWallets();
  const { ready: evmReady, wallets: evmWallets } = useWallets();

  const solanaWallet = solanaWallets.find(
    (wallet) => wallet.type === "solana" && wallet.walletClientType === "privy",
  );

  const evmWallet = evmWallets.find(
    (wallet) =>
      wallet.type === "ethereum" && wallet.walletClientType === "privy",
  );

  return useQuery<WalletAddresses | null, Error>({
    queryKey: ["privy-wallet"],
    queryFn: () => {
      if (!solanaWallet || !evmWallet) {
        return null;
      }
      return {
        solanaWallet: solanaWallet.address,
        evmWallet: evmWallet.address,
      };
    },
    enabled: solanaReady && evmReady && !!solanaWallet && !!evmWallet,
    staleTime: 20000,
    refetchInterval: 20000,
  });
};

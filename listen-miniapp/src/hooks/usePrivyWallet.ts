import { useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { useQuery } from "@tanstack/react-query";

interface WalletAddresses {
  solanaWalletAddress: string | null;
  evmWalletAddress: string | null;
}

export const usePrivyWallets = () => {
  const { ready: solanaReady, wallets: solanaWallets } = useSolanaWallets();
  const { ready: evmReady, wallets: evmWallets } = useWallets();

  const solanaWallet = solanaWallets.find(
    (wallet) => wallet.type === "solana" && wallet.walletClientType === "privy"
  );

  const evmWallet = evmWallets.find(
    (wallet) =>
      wallet.type === "ethereum" && wallet.walletClientType === "privy"
  );

  const { data } = useQuery<WalletAddresses | null, Error>({
    queryKey: ["privy-wallet"],
    queryFn: () => {
      const res = {
        solanaWalletAddress: solanaWallet?.address ?? null,
        evmWalletAddress: evmWallet?.address ?? null,
      };
      return res;
    },
    enabled: solanaReady && evmReady,
    staleTime: 20000,
    refetchInterval: 20000,
  });

  return {
    solanaWalletAddress: data?.solanaWalletAddress,
    evmWalletAddress: data?.evmWalletAddress,
    isLoadingSolana: solanaReady,
    isLoadingEvm: evmReady,
  };
};

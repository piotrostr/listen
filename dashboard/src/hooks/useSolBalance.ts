import { useQuery } from "@tanstack/react-query";
import { Connection, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { usePrivyWallet } from "./usePrivyWallet";

export const useSolBalance = () => {
  const connection = new Connection(import.meta.env.VITE_RPC_URL);
  const { data: pubkey } = usePrivyWallet();

  const fetchSOLBalance = async (): Promise<number> => {
    try {
      if (!pubkey) {
        throw new Error("No pubkey available");
      }

      const balance = await connection.getBalance(pubkey);
      return balance / LAMPORTS_PER_SOL;
    } catch (error) {
      console.error("Error fetching SOL balance:", error);
      throw error;
    }
  };

  return useQuery<number, Error>({
    queryKey: ["sol-balance"],
    queryFn: fetchSOLBalance,
    refetchInterval: 20000,
    staleTime: 20000,
    enabled: !!pubkey,
  });
};

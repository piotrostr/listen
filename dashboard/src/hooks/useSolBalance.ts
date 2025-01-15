import { useQuery } from "@tanstack/react-query";
import { Connection, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { usePubkey } from "./usePubkey";

export const useSolBalance = () => {
  const { data: pubkey } = usePubkey();
  const connection = new Connection(import.meta.env.VITE_RPC_URL);

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

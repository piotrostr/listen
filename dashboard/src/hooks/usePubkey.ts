import { useQuery } from "@tanstack/react-query";
import { PublicKey } from "@solana/web3.js";
import { useListen } from "./useListen";

export const usePubkey = () => {
  const { getPubkey } = useListen();

  const fetchPubkey = async (): Promise<PublicKey> => {
    try {
      const pubkeyResponse = await getPubkey();
      return new PublicKey(pubkeyResponse.pubkey);
    } catch (error) {
      console.error("Error fetching pubkey:", error);
      throw error;
    }
  };

  return useQuery<PublicKey, Error>({
    queryKey: ["pubkey"],
    queryFn: fetchPubkey,
  });
};

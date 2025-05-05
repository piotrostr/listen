import { useQuery } from "@tanstack/react-query";
import { fetchListenMetadata } from "../lib/listen";

export function useListenMetadata(pubkey: string) {
  return useQuery({
    queryKey: ["tokenMetadata", pubkey],
    queryFn: () => fetchListenMetadata(pubkey),
    enabled: !!pubkey, // Only run the query if pubkey is provided
    staleTime: 1000 * 60 * 5, // Consider data fresh for 5 minutes
  });
}

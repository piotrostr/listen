import { useQuery } from "@tanstack/react-query";
import { TokenMetadataRaw } from "../types/metadata";

export async function fetchListenMetadata(
  pubkey: string
): Promise<TokenMetadataRaw> {
  const response = await fetch(
    `https://api.listen-rs.com/v1/adapter/metadata?mint=${pubkey}`
  );

  if (!response.ok) {
    const text = await response.text();
    console.log(text);
    throw new Error(text || response.statusText);
  }

  return response.json();
}

export function useListenMetadata(pubkey: string) {
  return useQuery({
    queryKey: ["tokenMetadata", pubkey],
    queryFn: () => fetchListenMetadata(pubkey),
    enabled: !!pubkey, // Only run the query if pubkey is provided
    staleTime: 1000 * 60 * 5, // Consider data fresh for 5 minutes
  });
}

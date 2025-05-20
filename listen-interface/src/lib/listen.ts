import { config } from "../config";
import { TokenMetadataRaw } from "../types/metadata";

export async function fetchListenMetadata(
  pubkey: string
): Promise<TokenMetadataRaw> {
  const response = await fetch(
    `${config.adapterEndpoint}/metadata?mint=${pubkey}`
  );

  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || response.statusText);
  }

  return response.json();
}

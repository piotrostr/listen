import { COIN_DATA_URL } from "./consts";

export async function grabToken(mint: string) {
  const res = await fetch(COIN_DATA_URL + mint);
  const json = await res.json();
  return json;
}

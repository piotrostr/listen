import { Connection, ParsedTransactionWithMeta } from "@solana/web3.js";
import { fetchListenMetadata } from "./listen";
import { WSOL_MINT } from "./price";
import { formatAmountUI } from "./util";

interface BalanceChange {
  symbol: string;
  pubkey: string;
  uiAmount: string;
}

const parseBalanceChanges = (
  tx: ParsedTransactionWithMeta,
  userAddress: string
): BalanceChange => {
  const postTokenBalances = tx.meta?.postTokenBalances || [];
  const preTokenBalances = tx.meta?.preTokenBalances || [];

  // Find the balance change for the user
  for (let i = 0; i < postTokenBalances.length; i++) {
    const postBalance = postTokenBalances[i];

    if (postBalance.owner === userAddress) {
      // Find matching pre-balance
      const preBalance = preTokenBalances.find(
        (pre) => pre.mint === postBalance.mint
      );

      const postAmount = Number(postBalance.uiTokenAmount.uiAmount || 0);
      const preAmount = Number(preBalance?.uiTokenAmount.uiAmount || 0);
      const change = postAmount - preAmount;

      // Only return positive changes
      if (change > 0) {
        return {
          symbol: "",
          pubkey: postBalance.mint,
          uiAmount: formatAmountUI(change),
        };
      }
    }
  }

  // Check for SOL balance changes
  const preSOL =
    tx.meta?.preBalances.find(
      (_, index) =>
        tx.transaction.message.accountKeys[index].pubkey.toString() ===
        userAddress
    ) || 0;

  const postSOL =
    tx.meta?.postBalances.find(
      (_, index) =>
        tx.transaction.message.accountKeys[index].pubkey.toString() ===
        userAddress
    ) || 0;

  const solChange = (postSOL - preSOL) / 1e9;
  if (solChange > 0) {
    return {
      symbol: "SOL",
      pubkey: WSOL_MINT,
      uiAmount: formatAmountUI(solChange),
    };
  }

  return { symbol: "", pubkey: "", uiAmount: "0" };
};

export const getBalanceChange = async (
  signature: string,
  userAddress: string
): Promise<BalanceChange> => {
  const connection = new Connection(import.meta.env.VITE_RPC_URL, {
    commitment: "finalized",
  });
  const tx = await connection.getParsedTransaction(signature, {
    commitment: "finalized",
    maxSupportedTransactionVersion: 0,
  });

  if (!tx) {
    return { symbol: "", pubkey: "", uiAmount: "0" };
  }

  const change = parseBalanceChanges(tx, userAddress);
  const tokenThatChanged = await fetchListenMetadata(change.pubkey);

  return {
    symbol: tokenThatChanged.mpl.symbol,
    pubkey: change.pubkey,
    uiAmount: change.uiAmount,
  };
};

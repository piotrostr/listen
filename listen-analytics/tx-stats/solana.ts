import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Connection, PublicKey } from "@solana/web3.js";
import { write } from "bun";
import { type Wallet } from "./types";

// USDC token address on Solana mainnet
const USDC_MINT = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

// Interface for transaction count results
export interface WalletTransactionResult {
  wallet: Wallet;
  count: number;
  solBalance: number;
  usdcBalance: number;
  success: boolean;
  transactions?: {
    signature: string;
    blockTime: number;
  }[];
}

/**
 * Counts transactions and checks balances for a single Solana wallet
 */
export const countTransactionsForWallet = async (
  wallet: Wallet,
  connection: Connection
): Promise<WalletTransactionResult> => {
  try {
    const pubkey = new PublicKey(wallet.address);

    // Get transaction signatures with block time
    const transactions = await connection.getSignaturesForAddress(pubkey, {
      limit: 1000,
    });

    // Get SOL balance (in lamports, 1 SOL = 1,000,000,000 lamports)
    const solBalanceInLamports = await connection.getBalance(pubkey);
    const solBalance = solBalanceInLamports / 1_000_000_000;

    // Get USDC balance
    let usdcBalance = 0;
    try {
      // Find all token accounts owned by this wallet
      const tokenAccounts = await connection.getParsedTokenAccountsByOwner(
        pubkey,
        {
          programId: TOKEN_PROGRAM_ID,
        }
      );

      // Find USDC account if it exists
      const usdcAccount = tokenAccounts.value.find(
        (account) =>
          account.account.data.parsed.info.mint === USDC_MINT.toString()
      );

      if (usdcAccount) {
        // USDC has 6 decimals
        usdcBalance =
          Number(usdcAccount.account.data.parsed.info.tokenAmount.amount) /
          1_000_000;
      }
    } catch (tokenError) {
      console.warn(
        `Could not fetch USDC balance for ${wallet.address}:`,
        tokenError
      );
    }

    // Extract transaction data with block times
    const transactionData = transactions.map((tx) => ({
      signature: tx.signature,
      blockTime: tx.blockTime || 0,
    }));

    return {
      wallet,
      count: transactions.length,
      solBalance,
      usdcBalance,
      success: true,
      transactions: transactionData,
    };
  } catch (error) {
    console.error(`Error fetching data for ${wallet.address}:`, error);
    return {
      wallet,
      count: 0,
      solBalance: 0,
      usdcBalance: 0,
      success: false,
    };
  }
};

/**
 * Process wallets in chunks to control concurrency
 */
export const processWalletsInChunks = async (
  wallets: Wallet[],
  chunkSize: number,
  connection: Connection
): Promise<WalletTransactionResult[]> => {
  const results: WalletTransactionResult[] = [];

  // Process wallets in chunks
  for (let i = 0; i < wallets.length; i += chunkSize) {
    console.log(
      `Processing chunk ${i / chunkSize + 1} of ${Math.ceil(
        wallets.length / chunkSize
      )}`
    );

    const chunk = wallets.slice(i, i + chunkSize);
    const chunkResults = await Promise.all(
      chunk.map((wallet) => countTransactionsForWallet(wallet, connection))
    );

    results.push(...chunkResults);

    // Log progress
    const processedCount = Math.min(i + chunkSize, wallets.length);
    console.log(`Processed ${processedCount}/${wallets.length} wallets`);
  }

  return results;
};

/**
 * Process Solana wallets and generate transaction report
 */
export const processSolanaWallets = async (
  solanaWallets: Wallet[],
  connection: Connection,
  concurrencyLimit = 30
): Promise<WalletTransactionResult[]> => {
  console.log(
    `Starting to process ${solanaWallets.length} Solana wallets with concurrency limit of ${concurrencyLimit}`
  );

  const results = await processWalletsInChunks(
    solanaWallets,
    concurrencyLimit,
    connection
  );

  // write to file
  await write(
    "output/solana_transactions.json",
    JSON.stringify(results, null, 2)
  );

  // Generate and display report
  generateSolanaReport(results);

  // 200ms timeout
  await new Promise((resolve) => setTimeout(resolve, 200));

  return results;
};

/**
 * Generate and display a report of Solana wallet transaction data
 */
export const generateSolanaReport = (
  results: WalletTransactionResult[]
): void => {
  const successfulRequests = results.filter((r) => r.success);
  const totalTransactions = successfulRequests.reduce(
    (sum, result) => sum + result.count,
    0
  );
  const walletsWithTransactions = successfulRequests.filter((r) => r.count > 0);

  // Calculate balance totals
  const totalSolBalance = successfulRequests.reduce(
    (sum, result) => sum + result.solBalance,
    0
  );

  const totalUsdcBalance = successfulRequests.reduce(
    (sum, result) => sum + result.usdcBalance,
    0
  );

  // Find wallet with highest balances
  let highestSolWallet = successfulRequests[0];
  let highestUsdcWallet = successfulRequests[0];

  for (const result of successfulRequests) {
    if (result.solBalance > highestSolWallet.solBalance) {
      highestSolWallet = result;
    }
    if (result.usdcBalance > highestUsdcWallet.usdcBalance) {
      highestUsdcWallet = result;
    }
  }

  // Time series analysis
  const allTransactions = successfulRequests
    .flatMap((result) => result.transactions || [])
    .filter((tx) => tx.blockTime > 0)
    .sort((a, b) => a.blockTime - b.blockTime);

  // Group transactions by day
  const transactionsByDay = new Map<string, number>();
  allTransactions.forEach((tx) => {
    const date = new Date(tx.blockTime * 1000).toISOString().split("T")[0];
    transactionsByDay.set(date, (transactionsByDay.get(date) || 0) + 1);
  });

  // Calculate daily statistics
  const dailyStats = Array.from(transactionsByDay.entries())
    .sort(([dateA], [dateB]) => dateA.localeCompare(dateB))
    .map(([date, count]) => ({ date, count }));

  console.log(`
Processing complete:
- Successfully processed: ${successfulRequests.length}/${results.length} wallets
- Failed requests: ${results.length - successfulRequests.length}
- Wallets with transactions: ${walletsWithTransactions.length}
- Total transactions found: ${totalTransactions}
- Average transactions per wallet: ${(
    totalTransactions / successfulRequests.length
  ).toFixed(2)}
- Total SOL balance: ${totalSolBalance.toFixed(4)} SOL
- Total USDC balance: ${totalUsdcBalance.toFixed(2)} USDC
- Highest SOL balance: ${highestSolWallet.solBalance.toFixed(4)} SOL (${
    highestSolWallet.wallet.address
  })
- Highest USDC balance: ${highestUsdcWallet.usdcBalance.toFixed(2)} USDC (${
    highestUsdcWallet.wallet.address
  })

Time Series Analysis:
- First transaction: ${
    allTransactions.length > 0
      ? new Date(allTransactions[0].blockTime * 1000).toISOString()
      : "N/A"
  }
- Last transaction: ${
    allTransactions.length > 0
      ? new Date(
          allTransactions[allTransactions.length - 1].blockTime * 1000
        ).toISOString()
      : "N/A"
  }
- Daily transaction counts:
${dailyStats
  .map((stat) => `  ${stat.date}: ${stat.count} transactions`)
  .join("\n")}
`);

  // Write time series data to a separate file
  write("output/solana_time_series.json", JSON.stringify(dailyStats, null, 2));
};

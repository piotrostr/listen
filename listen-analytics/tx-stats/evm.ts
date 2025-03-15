import { Alchemy, AssetTransfersCategory, Network } from "alchemy-sdk";
import { write } from "bun";
import { type Wallet } from "./types";

// Interface for transaction count results
export interface EVMTransactionResult {
  wallet: Wallet;
  count: number;
  success: boolean;
  network: string;
}

// Configure networks with their supported transfer categories
const NETWORKS = {
  arbitrum: {
    network: Network.ARB_MAINNET,
    supportedCategories: [AssetTransfersCategory.EXTERNAL],
  },
  base: {
    network: Network.BASE_MAINNET,
    supportedCategories: [AssetTransfersCategory.EXTERNAL],
  },
};

// Create Alchemy instances for each network
const createAlchemyInstances = (apiKey: string) => {
  const instances: Record<
    string,
    { client: Alchemy; supportedCategories: AssetTransfersCategory[] }
  > = {};

  for (const [networkName, config] of Object.entries(NETWORKS)) {
    instances[networkName] = {
      client: new Alchemy({
        apiKey,
        network: config.network,
      }),
      supportedCategories: config.supportedCategories,
    };
  }

  return instances;
};

/**
 * Alternative implementation using getTransactionCount
 * This method is more reliable across all networks but only counts outgoing transactions
 */
export const countEVMTransactions = async (
  wallet: Wallet,
  alchemyData: {
    client: Alchemy;
    supportedCategories: AssetTransfersCategory[];
  },
  networkName: string
): Promise<EVMTransactionResult> => {
  try {
    // Get the transaction count (nonce) for the address
    const txCount = await alchemyData.client.core.getTransactionCount(
      wallet.address,
      "latest"
    );

    // Optionally, we could try to count incoming transactions too
    // But this is more complex and may hit rate limits

    return {
      wallet,
      count: txCount,
      success: true,
      network: networkName,
    };
  } catch (error) {
    console.error(
      `Error fetching transaction count for ${wallet.address} on ${networkName}:`,
      error
    );
    return {
      wallet,
      count: 0,
      success: false,
      network: networkName,
    };
  }
};

/**
 * Sleep function for rate limiting
 */
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Process wallets in chunks with rate limiting
 */
export const processEVMWalletsInChunks = async (
  wallets: Wallet[],
  chunkSize: number,
  alchemyInstances: Record<
    string,
    { client: Alchemy; supportedCategories: AssetTransfersCategory[] }
  >,
  requestsPerSecond: number = 10
): Promise<EVMTransactionResult[]> => {
  const results: EVMTransactionResult[] = [];
  const delayBetweenRequests = 1000 / requestsPerSecond;

  // Process wallets in chunks
  for (let i = 0; i < wallets.length; i += chunkSize) {
    console.log(
      `Processing EVM chunk ${Math.floor(i / chunkSize) + 1} of ${Math.ceil(
        wallets.length / chunkSize
      )}`
    );

    const chunk = wallets.slice(i, i + chunkSize);

    // Process each wallet in the chunk with rate limiting
    for (const wallet of chunk) {
      // Determine which network to use based on wallet chain data if available
      // Default to Ethereum mainnet wallet.chain_type.toLowerCase() || defaultNetwork;
      for (const network in alchemyInstances) {
        const startTime = Date.now();
        const result = await countEVMTransactions(
          wallet,
          alchemyInstances[network],
          network
        );
        results.push(result);
        // Calculate how long to wait to maintain rate limit
        const processingTime = Date.now() - startTime;
        const waitTime = Math.max(0, delayBetweenRequests - processingTime);

        if (waitTime > 0) {
          await sleep(waitTime);
        }
      }
    }

    // Log progress
    const processedCount = Math.min(i + chunkSize, wallets.length);
    console.log(`Processed ${processedCount}/${wallets.length} EVM wallets`);
  }

  return results;
};

/**
 * Process EVM wallets and generate transaction report
 */
export const processEVMWallets = async (
  evmWallets: Wallet[],
  alchemyApiKey: string,
  concurrencyChunkSize = 10
): Promise<EVMTransactionResult[]> => {
  console.log(
    `Starting to process ${evmWallets.length} EVM wallets with rate limit of 10 requests/second`
  );

  // Create Alchemy instances
  const alchemyInstances = createAlchemyInstances(alchemyApiKey);

  const results = await processEVMWalletsInChunks(
    evmWallets,
    concurrencyChunkSize,
    alchemyInstances
  );

  // Write to file
  await write("output/evm_transactions.json", JSON.stringify(results, null, 2));

  // Generate and display report
  generateEVMReport(results);

  return results;
};

/**
 * Generate and display a report of EVM wallet transaction data
 */
export const generateEVMReport = (results: EVMTransactionResult[]): void => {
  const successfulRequests = results.filter((r) => r.success);
  const totalTransactions = successfulRequests.reduce(
    (sum, result) => sum + result.count,
    0
  );
  const walletsWithTransactions = successfulRequests.filter((r) => r.count > 0);

  // Group by network
  const networkStats: Record<string, { count: number; transactions: number }> =
    {};

  for (const result of successfulRequests) {
    if (!networkStats[result.network]) {
      networkStats[result.network] = { count: 0, transactions: 0 };
    }
    networkStats[result.network].count++;
    networkStats[result.network].transactions += result.count;
  }

  console.log(`
EVM Processing complete:
- Successfully processed: ${successfulRequests.length}/${results.length} wallets
- Failed requests: ${results.length - successfulRequests.length}
- Wallets with transactions: ${walletsWithTransactions.length}
- Total transactions found: ${totalTransactions}
- Average transactions per wallet: ${(
    totalTransactions / successfulRequests.length
  ).toFixed(2)}

Network breakdown:`);

  for (const [network, stats] of Object.entries(networkStats)) {
    console.log(
      `- ${network}: ${stats.count} wallets, ${stats.transactions} transactions`
    );
  }
};

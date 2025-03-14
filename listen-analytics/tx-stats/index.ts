import { Connection } from "@solana/web3.js";
import { file } from "bun";
import { processEVMWallets } from "./evm";
import { processSolanaWallets } from "./solana";
import { type Wallet, WalletResponseSchema } from "./types";

const allWallets: Wallet[] = [];

// 10 pages total we got
for (let page = 1; page <= 10; page++) {
  const data = await file(`output/wallets_page_${page}.json`).json();

  const wallets = WalletResponseSchema.parse(data);

  allWallets.push(...wallets.data);
}

console.log("all: ", allWallets.length);

const solanaWallets = allWallets.filter(
  (wallet) => wallet.chain_type === "solana",
);

console.log("solana: ", solanaWallets.length);

const ethereumWallets = allWallets.filter(
  (wallet) => wallet.chain_type === "ethereum",
);

console.log("ethereum: ", ethereumWallets.length);

// Initialize Solana connection
const connection = new Connection(process.env.SOLANA_RPC_URL!);

// Process Solana wallets
await processSolanaWallets(solanaWallets, connection);

// Process Ethereum and other EVM wallets
await processEVMWallets(ethereumWallets, process.env.ALCHEMY_API_KEY!);

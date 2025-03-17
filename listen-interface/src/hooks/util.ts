import { User, WalletWithMetadata } from "@privy-io/react-auth";
import { PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import { getAddress } from "viem";
import ethLogo from "../assets/icons/ethereum.svg";
import {
  Pipeline,
  PipelineActionType,
  PipelineConditionType,
  PipelineSchema,
} from "../types/pipeline";
import { PortfolioItem } from "./types";

interface RawAccount {
  mint: PublicKey;
  owner: PublicKey;
  amount: bigint;
  delegateOption: number;
  delegate: PublicKey;
  state: number;
  isNativeOption: number;
  isNative: bigint;
  delegatedAmount: bigint;
  closeAuthorityOption: number;
  closeAuthority: PublicKey;
}

export function decodeTokenAccount(data: Buffer): RawAccount {
  return {
    mint: new PublicKey(data.slice(0, 32)),
    owner: new PublicKey(data.slice(32, 64)),
    amount: data.readBigUInt64LE(64),
    delegateOption: data.readUInt32LE(72),
    delegate: new PublicKey(data.slice(76, 108)),
    state: data[108],
    isNativeOption: data.readUInt32LE(109),
    isNative: data.readBigUInt64LE(113),
    delegatedAmount: data.readBigUInt64LE(121),
    closeAuthorityOption: data.readUInt32LE(129),
    closeAuthority: new PublicKey(data.slice(133, 165)),
  };
}

export const userHasDelegatedSolanaWallet = (user: User | null) => {
  return !!user?.linkedAccounts.find(
    (account): account is WalletWithMetadata =>
      account.type === "wallet" &&
      account.delegated &&
      account.chainType === "solana"
  );
};

export const userHasDelegatedEvmWallet = (user: User | null) => {
  return !!user?.linkedAccounts.find(
    (account): account is WalletWithMetadata =>
      account.type === "wallet" &&
      account.delegated &&
      account.chainType === "ethereum"
  );
};

export const imageMap = {
  "0xaf88d065e77c8cc2239327c5edb3a432268e5831":
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png",
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v:
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png",
  So11111111111111111111111111111111111111112:
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
  solana:
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
  eth: ethLogo,
  ethereum: ethLogo,
  arb: "https://arbiscan.io/assets/arbitrum/images/svg/logos/chain-light.svg?v=25.1.4.0",
  "11111111111111111111111111111111":
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
};

export const caip2Map = {
  solana: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
  ethereum: "eip155:1",
  bsc: "eip155:56",
  arbitrum: "eip155:42161",
  base: "eip155:8453",
  blast: "eip155:81457",
  avalanche: "eip155:43114",
  polygon: "eip155:137",
  scroll: "eip155:534352",
  optimism: "eip155:10",
  linea: "eip155:59144",
  gnosis: "eip155:100",
  fantom: "eip155:250",
  moonriver: "eip155:1285",
  moonbeam: "eip155:1284",
  boba: "eip155:288",
  mode: "eip155:34443",
  metis: "eip155:1088",
  lisk: "eip155:1135",
  aurora: "eip155:1313161554",
  sei: "eip155:1329",
  immutability: "eip155:13371",
  gravity: "eip155:1625",
  taiko: "eip155:167000",
  cronos: "eip155:25",
  fraxtal: "eip155:252",
  abstract: "eip155:2741",
  celo: "eip155:42220",
  world: "eip155:480",
  mantle: "eip155:5000",
  berachain: "eip155:80094",
};

// add more here, the stuff that is not easily searchable and needs to be spot on
export const addressBook = {
  solana: {
    SOL: "So11111111111111111111111111111111111111112",
    USDC: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    BONK: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    LISTEN: "Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump",
    JLP: "27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4",
  },
  ethereum: {
    usdc: getAddress("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    pepe: getAddress("0x25d887Ce7a35172C62FeBFD67a1856F20FaEbB00"),
  },
  arbitrum: {
    usdc: getAddress("0xaf88d065e77c8cC2239327C5EDb3A432268e5831"),
  },
  base: {
    usdc: getAddress("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
  },
  avalanche: {
    usdc: getAddress("0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E"),
  },
  bsc: {
    usdc: getAddress("0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d"),
  },
};

export const formatAmount = (amount: string, decimals: number) => {
  const amountNum = parseFloat(amount);
  return (amountNum / Math.pow(10, decimals)).toFixed(5).toString();
};

export function serializePipeline(pipeline: Pipeline): string {
  return JSON.stringify(pipeline);
}

export function deserializePipeline(serialized: string): Pipeline {
  const parsed = JSON.parse(serialized);
  return PipelineSchema.parse(parsed);
}

export const mockOrderPipeline: Pipeline = {
  steps: [
    {
      action: {
        type: PipelineActionType.SwapOrder,
        input_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        output_token: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        amount: "1000000000000000000",
        from_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        to_chain_caip2: "eip155:1",
      },
      conditions: [],
    },
    {
      action: {
        type: PipelineActionType.SwapOrder,
        input_token: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        output_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        amount: "1000000000000000000",
        from_chain_caip2: "eip155:1",
        to_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
      },
      conditions: [
        {
          type: PipelineConditionType.PriceAbove,
          asset: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
          value: 0.052,
        },
      ],
    },
  ],
};

export const caip2ToChainIdMap = {
  "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp": "solana",
  "eip155:1": "ethereum",
  "eip155:56": "bsc",
  "eip155:42161": "arbitrum",
  "eip155:8453": "base",
  "eip155:81457": "blast",
  "eip155:43114": "avalanche",
  "eip155:137": "polygon",
  "eip155:534352": "scroll",
  "eip155:10": "optimism",
  "eip155:59144": "linea",
  "eip155:100": "gnosis",
  "eip155:250": "fantom",
  "eip155:1285": "moonriver",
  "eip155:1284": "moonbeam",
  "eip155:288": "boba",
  "eip155:34443": "mode",
  "eip155:1088": "metis",
  "eip155:1135": "lisk",
  "eip155:1313161554": "aurora",
  "eip155:1329": "sei",
  "eip155:13371": "immutability",
  "eip155:1625": "gravity",
  "eip155:167000": "taiko",
  "eip155:25": "cronos",
  "eip155:252": "fraxtal",
  "eip155:2741": "abstract",
  "eip155:42220": "celo",
  "eip155:480": "world",
  "eip155:5000": "manta",
  "eip155:80094": "berachain",
};

export const caip2ToChainId = (caip2: string) => {
  if (caip2 in caip2ToChainIdMap) {
    return caip2ToChainIdMap[caip2 as keyof typeof caip2ToChainIdMap];
  }
  return null;
};

export const caip2ToChainIdNumericMap: { [key: string]: number } = {
  "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp": 1151111081099710,
  "eip155:1": 1, // ethereum
  "eip155:56": 56, // bsc
  "eip155:42161": 42161, // arbitrum
  "eip155:8453": 8453, // base
  "eip155:81457": 81457, // blast
  "eip155:43114": 43114, // avalanche
  "eip155:137": 137, // polygon
  "eip155:534352": 534352, // scroll
  "eip155:10": 10, // optimism
  "eip155:59144": 59144, // linea
  "eip155:100": 100, // gnosis
  "eip155:250": 250, // fantom
  "eip155:1285": 1285, // moonriver
  "eip155:1284": 1284, // moonbeam
  "eip155:288": 288, // boba
  "eip155:34443": 34443, // mode
  "eip155:1088": 1088, // metis
  "eip155:1135": 1135, // lisk
  "eip155:1313161554": 1313161554, // aurora
  "eip155:1329": 1329, // sei
  "eip155:13371": 13371, // immutable
  "eip155:1625": 1625, // gravity
  "eip155:167000": 167000, // taiko
  "eip155:25": 25, // cronos
  "eip155:252": 252, // fraxtal
  "eip155:2741": 2741, // abstract
  "eip155:42220": 42220, // celo
  "eip155:480": 480, // world
  "eip155:5000": 5000, // mantle
  "eip155:80094": 80094, // berachain
};

export const caip2ToLifiChainId = (caip2: string): number => {
  return caip2ToChainIdNumericMap[
    caip2 as keyof typeof caip2ToChainIdNumericMap
  ];
};

export const chainIdNumericToChainId = (chainId: number): string => {
  if (chainId === 1151111081099710) {
    return "solana";
  }
  const key = `eip155:${chainId}`;
  return caip2ToChainIdMap[key as keyof typeof caip2ToChainIdMap];
};

// Validate Solana transaction signatures
export const isValidSolanaTransactionSignature = (
  signature: string
): boolean => {
  try {
    // Check that it only contains valid base58 characters
    const base58Regex = /^[1-9A-HJ-NP-Za-km-z]+$/;
    if (!base58Regex.test(signature)) {
      return false;
    }

    // Decode the base58 string to get the actual bytes
    const bytes = bs58.decode(signature);

    // Solana transaction signatures should be exactly 64 bytes
    return bytes.length === 64;
  } catch {
    return false;
  }
};

// Validate Solana addresses
export const isValidSolanaAddress = (address: string): boolean => {
  try {
    new PublicKey(address);
    return true;
  } catch {
    return false;
  }
};

// Validate EVM addresses
export const isValidEvmAddress = (address: string): boolean => {
  // EVM addresses are 42 characters long (0x + 40 hex characters)
  const evmAddressRegex = /^0x[a-fA-F0-9]{40}$/;
  return evmAddressRegex.test(address);
};

// Validate EVM transaction hashes
export const isValidEvmTransaction = (hash: string): boolean => {
  // EVM transaction hashes are 66 characters long (0x + 64 hex characters)
  const evmTxRegex = /^0x[a-fA-F0-9]{64}$/;
  return evmTxRegex.test(hash);
};

export const renderTimestamps = (text: string): string => {
  const timestampPattern = /(\d{10})/g;
  let match;

  while ((match = timestampPattern.exec(text)) !== null) {
    const timestamp = match[1];
    const date = new Date(parseInt(timestamp) * 1000);
    const formattedDate = date.toLocaleString();
    text = text.replace(match[0], formattedDate);
  }

  return text;
};

// Render addresses and transaction hashes as links
export const renderAddressOrTx = (text: string): string => {
  if (!text) return "";

  // Process the text by replacing matches with HTML links
  let processedText = text;

  // Handle backtick-enclosed addresses and transactions
  const backtickPattern = /`([^`]+)`/g;
  let backtickMatch;

  while ((backtickMatch = backtickPattern.exec(text)) !== null) {
    const fullMatch = backtickMatch[0]; // The entire match including backticks
    const content = backtickMatch[1]; // Just the content part (without backticks)

    // Check if the content is a valid address or transaction
    const isSolanaAddress = isValidSolanaAddress(content);
    const isSolanaTx = isValidSolanaTransactionSignature(content);
    const isEvmAddress = isValidEvmAddress(content);
    const isEvmTx = isValidEvmTransaction(content);

    if (isSolanaAddress || isSolanaTx || isEvmAddress || isEvmTx) {
      let url;
      let displayText;

      if (isSolanaTx) {
        url = `https://solscan.io/tx/${content}`;
        displayText = `${content.slice(0, 4)}..${content.slice(-4)}`;
      } else if (isSolanaAddress) {
        url = `https://solscan.io/address/${content}`;
        displayText = `${content.slice(0, 4)}..${content.slice(-4)}`;
      } else if (isEvmTx) {
        url = `https://blockscan.com/tx/${content}`;
        displayText = content;
      } else {
        // isEvmAddress
        url = `https://blockscan.com/address/${content}`;
        displayText = content;
      }

      // Create the replacement with the link (without backticks)
      const replacement = `<a href="${url}" target="_blank" rel="noopener noreferrer" class="text-blue-400 underline">${displayText}</a>`;

      // Replace this specific occurrence
      processedText =
        processedText.substring(0, backtickMatch.index) +
        replacement +
        processedText.substring(backtickMatch.index + fullMatch.length);

      // Adjust the regex lastIndex to account for the replacement
      backtickPattern.lastIndex += replacement.length - fullMatch.length;
    }
  }

  // Special case for quoted transaction signatures - they need a more specific pattern
  const quotedTxPattern = /"([1-9A-HJ-NP-Za-km-z]{87,88})"/g;
  let quotedMatch;

  while ((quotedMatch = quotedTxPattern.exec(processedText)) !== null) {
    const fullMatch = quotedMatch[0]; // The entire match including quotes
    const txSignature = quotedMatch[1]; // Just the signature part (without quotes)

    if (isValidSolanaTransactionSignature(txSignature)) {
      const url = `https://solscan.io/tx/${txSignature}`;

      // Create the replacement with the link
      const replacement = `"<a href="${url}" target="_blank" rel="noopener noreferrer" class="text-blue-400 underline">${txSignature.slice(
        0,
        4
      )}..${txSignature.slice(-4)}</a>"`;

      // Replace this specific occurrence
      processedText =
        processedText.substring(0, quotedMatch.index) +
        replacement +
        processedText.substring(quotedMatch.index + fullMatch.length);

      // Adjust the regex lastIndex to account for the replacement
      quotedTxPattern.lastIndex += replacement.length - fullMatch.length;
    }
  }

  // Add a specific pattern for non-quoted Solana transaction signatures (87-88 chars)
  const longSolanaTxPattern = /\b([1-9A-HJ-NP-Za-km-z]{87,88})\b/g;
  let longTxMatch;

  while ((longTxMatch = longSolanaTxPattern.exec(processedText)) !== null) {
    const fullMatch = longTxMatch[0];

    // Skip if this is already inside an HTML tag (from previous replacements)
    const prevText = processedText.substring(
      Math.max(0, longTxMatch.index - 50),
      longTxMatch.index
    );
    if (prevText.includes('<a href="https://solscan.io/')) {
      continue;
    }

    if (isValidSolanaTransactionSignature(fullMatch)) {
      const url = `https://solscan.io/tx/${fullMatch}`;

      // Create the replacement with the link
      const replacement = `<a href="${url}" target="_blank" rel="noopener noreferrer" class="text-blue-400 underline">${fullMatch.slice(
        0,
        4
      )}..${fullMatch.slice(-4)}</a>`;

      // Replace this specific occurrence
      processedText =
        processedText.substring(0, longTxMatch.index) +
        replacement +
        processedText.substring(longTxMatch.index + fullMatch.length);

      // Adjust the regex lastIndex to account for the replacement
      longSolanaTxPattern.lastIndex += replacement.length - fullMatch.length;
    }
  }

  // Handle regular Solana addresses and transactions
  const solanaPattern = /\b([1-9A-HJ-NP-Za-km-z]{32,44})\b/g;
  let match;

  // Use a while loop with exec to get all matches with their positions
  while ((match = solanaPattern.exec(processedText)) !== null) {
    const fullMatch = match[0]; // The entire match

    // Skip if this is already inside an HTML tag (from previous replacements)
    const prevText = processedText.substring(
      Math.max(0, match.index - 50),
      match.index
    );
    if (prevText.includes('<a href="https://solscan.io/')) {
      continue;
    }

    // Determine if it's a Solana address or transaction
    const isSolanaAddress = isValidSolanaAddress(fullMatch);
    const isSolanaTx = isValidSolanaTransactionSignature(fullMatch);

    if (isSolanaAddress || isSolanaTx) {
      const url = isSolanaTx
        ? `https://solscan.io/tx/${fullMatch}`
        : `https://solscan.io/address/${fullMatch}`;

      // Create the replacement with the link
      const replacement = `<a href="${url}" target="_blank" rel="noopener noreferrer" class="text-blue-400 underline">${fullMatch.slice(
        0,
        4
      )}..${fullMatch.slice(-4)}</a>`;

      // Replace this specific occurrence
      processedText =
        processedText.substring(0, match.index) +
        replacement +
        processedText.substring(match.index + fullMatch.length);

      // Adjust the regex lastIndex to account for the replacement
      solanaPattern.lastIndex += replacement.length - fullMatch.length;
    }
  }

  // Now handle EVM addresses and transactions
  const evmPattern = /\b(0x[a-fA-F0-9]{40,64})\b/g;

  // Reset match for the new pattern
  match = null;

  while ((match = evmPattern.exec(processedText)) !== null) {
    const fullMatch = match[0]; // The entire match

    // Skip if this is already inside an HTML tag (from previous replacements)
    const prevText = processedText.substring(
      Math.max(0, match.index - 50),
      match.index
    );
    if (prevText.includes('<a href="https://blockscan.com/')) {
      continue;
    }

    // Determine if it's an EVM address or transaction
    const isEvmAddress = isValidEvmAddress(fullMatch);
    const isEvmTx = isValidEvmTransaction(fullMatch);

    if (isEvmAddress || isEvmTx) {
      const url = isEvmTx
        ? `https://blockscan.com/tx/${fullMatch}`
        : `https://blockscan.com/address/${fullMatch}`;

      // Create the replacement with the link
      const replacement = `<a href="${url}" target="_blank" rel="noopener noreferrer" class="text-blue-400 underline">${fullMatch}</a>`;

      // Replace this specific occurrence
      processedText =
        processedText.substring(0, match.index) +
        replacement +
        processedText.substring(match.index + fullMatch.length);

      // Adjust the regex lastIndex to account for the replacement
      evmPattern.lastIndex += replacement.length - fullMatch.length;
    }
  }

  return processedText;
};

export type CompactPortfolio = {
  chain: string;
  address: string;
  amount: string;
  name: string;
  symbol: string;
  decimals: number;
  value: string;
}[];

export const compactPortfolio = (
  portfolio: PortfolioItem[]
): CompactPortfolio => {
  return portfolio.map((token) => ({
    chain: token.chain,
    address: token.address,
    amount: token.amount.toString(),
    name: token.name,
    symbol: token.symbol,
    decimals: token.decimals,
    value: (token.amount * token.price).toFixed(2),
  }));
};

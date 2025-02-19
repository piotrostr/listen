import { User, WalletWithMetadata } from "@privy-io/react-auth";
import { PublicKey } from "@solana/web3.js";
import ethLogo from "../assets/icons/ethereum.svg";

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
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v:
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png",
  So11111111111111111111111111111111111111112:
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
  solana:
    "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png",
  eth: ethLogo,
  arb: "https://arbiscan.io/assets/arbitrum/images/svg/logos/chain-light.svg?v=25.1.4.0",
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
    solana: "So11111111111111111111111111111111111111112",
    usdc: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    bonk: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
  },
  ethereum: {
    usdc: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    pepe: "0x25d887Ce7a35172C62FeBFD67a1856F20FaEbB00",
  },
  arbitrum: {
    usdc: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  },
  base: {
    usdc: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
  },
  avalanche: {
    usdc: "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E",
  },
};

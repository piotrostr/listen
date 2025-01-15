import {
  BalanceResponse,
  HoldingsResponse,
  PriceResponse,
  PubkeyResponse,
  SwapResponse,
  PumpBuyParams,
  PumpSellParams,
  SwapParams,
  TokenBalanceParams,
  TokenBalanceResponse,
  BalanceResponseSchema,
  HoldingsResponseSchema,
  PriceResponseSchema,
  PubkeyResponseSchema,
  SwapResponseSchema,
  PumpBuyParamsSchema,
  PumpSellParamsSchema,
  SwapParamsSchema,
  TokenBalanceParamsSchema,
  TokenBalanceResponseSchema,
} from "./schema";

export interface UseListenActions {
  balance: (pubkey: string) => Promise<BalanceResponse>;
  getHoldings: () => Promise<HoldingsResponse>;
  getPrice: (mint: string) => Promise<PriceResponse>;
  getPubkey: () => Promise<PubkeyResponse>;
  pumpBuy: (params: PumpBuyParams) => Promise<SwapResponse>;
  pumpSell: (params: PumpSellParams) => Promise<SwapResponse>;
  swap: (params: SwapParams) => Promise<SwapResponse>;
  tokenBalance: (params: TokenBalanceParams) => Promise<TokenBalanceResponse>;
}

export function useListen(): UseListenActions {
  const API_BASE = "http://localhost:6969";

  const balance = async (pubkey: string): Promise<BalanceResponse> => {
    const response = await fetch(`${API_BASE}/balance`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pubkey }),
    });
    const data = await response.json();
    return BalanceResponseSchema.parse(data);
  };

  const getHoldings = async (): Promise<HoldingsResponse> => {
    const response = await fetch(`${API_BASE}/holdings`);
    const data = await response.json();
    return HoldingsResponseSchema.parse(data);
  };

  const getPrice = async (mint: string): Promise<PriceResponse> => {
    const response = await fetch(`${API_BASE}/price`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ mint }),
    });
    const data = await response.json();
    return PriceResponseSchema.parse(data);
  };

  const getPubkey = async (): Promise<PubkeyResponse> => {
    const response = await fetch(`${API_BASE}/pubkey`);
    const data = await response.json();
    return PubkeyResponseSchema.parse(data);
  };

  const pumpBuy = async (params: PumpBuyParams): Promise<SwapResponse> => {
    PumpBuyParamsSchema.parse(params);
    const response = await fetch(`${API_BASE}/pump-buy`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
    const data = await response.json();
    return SwapResponseSchema.parse(data);
  };

  const pumpSell = async (params: PumpSellParams): Promise<SwapResponse> => {
    PumpSellParamsSchema.parse(params);
    const response = await fetch(`${API_BASE}/pump-sell`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
    const data = await response.json();
    return SwapResponseSchema.parse(data);
  };

  const swap = async (params: SwapParams): Promise<SwapResponse> => {
    SwapParamsSchema.parse(params);
    const response = await fetch(`${API_BASE}/swap`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
    const data = await response.json();
    return SwapResponseSchema.parse(data);
  };

  const tokenBalance = async (
    params: TokenBalanceParams,
  ): Promise<TokenBalanceResponse> => {
    TokenBalanceParamsSchema.parse(params);
    const response = await fetch(`${API_BASE}/token_balance`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
    const data = await response.json();
    return TokenBalanceResponseSchema.parse(data);
  };

  return {
    balance,
    getHoldings,
    getPrice,
    getPubkey,
    pumpBuy,
    pumpSell,
    swap,
    tokenBalance,
  };
}

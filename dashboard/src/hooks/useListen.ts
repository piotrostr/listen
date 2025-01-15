import { useCallback } from "react";

export interface BalanceResponse {
  balance: number;
  pubkey: string;
}

export interface Holding {
  mint: string;
  ata: string;
  amount: number;
}

export interface HoldingsResponse {
  holdings: Array<Holding>;
}

export interface PriceResponse {
  mint: string;
  price: number;
}

export interface PumpBuyParams {
  mint: string;
  sol_amount: number;
  slippage: number;
}

export interface PumpSellParams {
  mint: string;
  token_amount: number;
  slippage: number;
}

export interface SwapParams {
  input_mint: string;
  output_mint: string;
  amount: number;
  slippage: number;
}

export interface TokenBalanceParams {
  pubkey: string;
  mint: string;
}

export interface TokenBalanceResponse {
  balance: number;
  mint: string;
  pubkey: string;
}

export interface PubkeyResponse {
  pubkey: string;
}

export interface UseListenActions {
  balance: (pubkey: string) => Promise<BalanceResponse>;
  getHoldings: () => Promise<HoldingsResponse>;
  getPrice: (mint: string) => Promise<PriceResponse>;
  getPubkey: () => Promise<PubkeyResponse>;
  pumpBuy: (params: PumpBuyParams) => Promise<void>;
  pumpSell: (params: PumpSellParams) => Promise<void>;
  swap: (params: SwapParams) => Promise<void>;
  tokenBalance: (params: TokenBalanceParams) => Promise<TokenBalanceResponse>;
}

// TODO some try catches here would be nice

export function useListen(): UseListenActions {
  const API_BASE = "http://localhost:6969"; // Adjust this to your API endpoint

  const balance = useCallback(async (pubkey: string) => {
    const response = await fetch(`${API_BASE}/balance`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pubkey }),
    });
    return response.json();
  }, []);

  const getHoldings = useCallback(async () => {
    const response = await fetch(`${API_BASE}/holdings`);
    return response.json();
  }, []);

  const getPrice = useCallback(async (mint: string) => {
    const response = await fetch(`${API_BASE}/price`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ mint }),
    });
    return response.json();
  }, []);

  const getPubkey = useCallback(async () => {
    const response = await fetch(`${API_BASE}/pubkey`);
    return await response.json();
  }, []);

  const pumpBuy = useCallback(
    async (params: { mint: string; sol_amount: number; slippage: number }) => {
      await fetch(`${API_BASE}/pump-buy`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(params),
      });
    },
    [],
  );

  const pumpSell = useCallback(
    async (params: {
      mint: string;
      token_amount: number;
      slippage: number;
    }) => {
      await fetch(`${API_BASE}/pump-sell`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(params),
      });
    },
    [],
  );

  const swap = useCallback(
    async (params: {
      input_mint: string;
      output_mint: string;
      amount: number;
      slippage: number;
    }) => {
      await fetch(`${API_BASE}/swap`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(params),
      });
    },
    [],
  );

  const tokenBalance = useCallback(
    async (params: { pubkey: string; mint: string }) => {
      const response = await fetch(`${API_BASE}/token_balance`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(params),
      });
      return response.json();
    },
    [],
  );

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

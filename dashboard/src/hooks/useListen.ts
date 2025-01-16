import { z } from "zod";
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

interface ApiError {
  status: number;
  message: string;
}

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

interface ApiError {
  status: number;
  message: string;
}

type FetchError = {
  status?: number;
  message?: string;
} & Error;

async function withRetry<T>(
  operation: () => Promise<T>,
  {
    initialDelay = 50,
    maxAttempts = 3,
    shouldRetry = (error: ApiError) => error.status === 500,
  } = {},
): Promise<T> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await operation();
    } catch (error) {
      const fetchError = error as FetchError;

      const apiError: ApiError = {
        status: fetchError.status ?? 500,
        message: fetchError.message ?? "Unknown error",
      };

      if (!shouldRetry(apiError) || attempt === maxAttempts) {
        throw apiError;
      }

      await wait(initialDelay * Math.pow(2, attempt - 1));
    }
  }

  throw new Error("Retry failed");
}

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

  const fetchWithRetry = async (
    url: string,
    options?: RequestInit,
  ): Promise<Response> => {
    return withRetry(async () => {
      const response = await fetch(`${API_BASE}${url}`, options);
      if (!response.ok) {
        throw {
          status: response.status,
          message: `HTTP error! status: ${response.status}`,
        };
      }
      return response;
    });
  };

  const makeRequest = async <T extends z.ZodType>(
    endpoint: string,
    schema: T,
    options?: RequestInit,
  ): Promise<z.infer<T>> => {
    const response = await fetchWithRetry(endpoint, options);
    const data = await response.json();
    return schema.parse(data);
  };

  // Transaction-sending operations with retry logic
  const pumpBuy = async (params: PumpBuyParams): Promise<SwapResponse> => {
    PumpBuyParamsSchema.parse(params);
    return makeRequest("/pump-buy", SwapResponseSchema, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
  };

  const pumpSell = async (params: PumpSellParams): Promise<SwapResponse> => {
    PumpSellParamsSchema.parse(params);
    return makeRequest("/pump-sell", SwapResponseSchema, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
  };

  const swap = async (params: SwapParams): Promise<SwapResponse> => {
    SwapParamsSchema.parse(params);
    return makeRequest("/swap", SwapResponseSchema, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
  };

  // Non-transaction operations (without retry)
  const balance = async (pubkey: string): Promise<BalanceResponse> => {
    return makeRequest("/balance", BalanceResponseSchema, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ pubkey }),
    });
  };

  const getHoldings = async (): Promise<HoldingsResponse> => {
    return makeRequest("/holdings", HoldingsResponseSchema, {
      method: "GET",
    });
  };

  const getPrice = async (mint: string): Promise<PriceResponse> => {
    return makeRequest("/price", PriceResponseSchema, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ mint }),
    });
  };

  const getPubkey = async (): Promise<PubkeyResponse> => {
    return makeRequest("/pubkey", PubkeyResponseSchema, {
      method: "GET",
    });
  };

  const tokenBalance = async (
    params: TokenBalanceParams,
  ): Promise<TokenBalanceResponse> => {
    TokenBalanceParamsSchema.parse(params);
    return makeRequest("/token_balance", TokenBalanceResponseSchema, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
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

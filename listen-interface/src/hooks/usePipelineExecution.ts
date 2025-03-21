import { usePrivy } from "@privy-io/react-auth";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { config } from "../config";
import { useToast } from "../contexts/ToastContext";
import {
  Pipeline,
  PipelineActionType,
  PipelineConditionType,
} from "../types/pipeline";
import { usePrivyWallets } from "./usePrivyWallet";

interface ExecuteOptions {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export function usePipelineExecution() {
  const [isExecuting, setIsExecuting] = useState(false);
  const { getAccessToken } = usePrivy();
  const { showToast } = useToast();
  const { data: wallets } = usePrivyWallets();
  const queryClient = useQueryClient();

  const { t } = useTranslation();

  const invalidateSolanaPortfolio = () => {
    if (wallets?.solanaWallet) {
      queryClient.refetchQueries({
        queryKey: ["portfolio", wallets.solanaWallet.toString()],
      });
    }
  };

  // Execute any pipeline
  const executePipeline = async (
    pipeline: Pipeline,
    options?: ExecuteOptions
  ) => {
    setIsExecuting(true);
    try {
      const token = await getAccessToken();
      const res = await fetch(config.engineEndpoint + "/pipeline", {
        method: "POST",
        body: JSON.stringify(pipeline),
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
      });

      invalidateSolanaPortfolio();

      if (!res.ok) {
        throw new Error("Failed to execute pipeline");
      }

      showToast(t("pipeline_execution.pipeline_scheduled"), "success");
      options?.onSuccess?.();
      return true;
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : t("pipeline_execution.error");
      showToast(errorMessage, "error");
      options?.onError?.(
        error instanceof Error ? error : new Error(errorMessage)
      );
      return false;
    } finally {
      setIsExecuting(false);
    }
  };

  // Quick buy a token with SOL
  const quickBuyToken = async (
    tokenAddress: string,
    solAmount: number,
    options?: ExecuteOptions
  ) => {
    const lamports = Math.floor(solAmount * LAMPORTS_PER_SOL).toString();

    const buyPipeline: Pipeline = {
      steps: [
        {
          action: {
            type: PipelineActionType.SwapOrder,
            input_token: "So11111111111111111111111111111111111111112", // wSOL
            output_token: tokenAddress,
            amount: lamports,
            from_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            to_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
          },
          conditions: [
            {
              type: PipelineConditionType.Now,
              asset: tokenAddress,
              value: 0, // Not used for Now condition
            },
          ],
        },
      ],
    };

    return executePipeline(buyPipeline, {
      onSuccess: () => {
        showToast(
          `${t("pipeline_execution.buy_order_placed")} ${tokenAddress}`,
          "success"
        );
        options?.onSuccess?.();
      },
      onError: (error) => {
        showToast(
          `${t("pipeline_execution.failed_to_buy_token")} ${error.message}`,
          "error"
        );
        options?.onError?.(error);
      },
    });
  };

  // Sell a token for SOL
  const sellTokenForSol = async (
    tokenAddress: string,
    tokenAmount: number,
    tokenDecimals: number,
    tokenName: string,
    options?: ExecuteOptions
  ) => {
    const rawAmount = Math.floor(tokenAmount * 10 ** tokenDecimals).toString();

    const sellPipeline: Pipeline = {
      steps: [
        {
          action: {
            type: PipelineActionType.SwapOrder,
            input_token: tokenAddress,
            output_token: "So11111111111111111111111111111111111111112", // wSOL
            amount: rawAmount,
            from_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            to_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
          },
          conditions: [
            {
              type: PipelineConditionType.Now,
              asset: tokenAddress,
              value: 0,
            },
          ],
        },
      ],
    };

    return executePipeline(sellPipeline, {
      onSuccess: () => {
        showToast(`Sell order placed for ${tokenName}`, "success");
        options?.onSuccess?.();
      },
      onError: (error) => {
        showToast(`Failed to sell token: ${error.message}`, "error");
        options?.onError?.(error);
      },
    });
  };

  return {
    isExecuting,
    executePipeline,
    quickBuyToken,
    sellTokenForSol,
  };
}

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
import { chainIdToCaip2 } from "./util";

interface ExecuteOptions {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
  chainId?: string;
}

export function usePipelineExecution() {
  const [isExecuting, setIsExecuting] = useState(false);
  const { getAccessToken } = usePrivy();
  const { showToast } = useToast();
  const queryClient = useQueryClient();

  const { t } = useTranslation();

  const triggerPipelinesRefetch = () => {
    console.log(
      "usePipelineExecution: Triggering immediate pipelines refetch."
    );
    queryClient.invalidateQueries({ queryKey: ["pipelines"] });
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

      if (!res.ok) {
        const errorData = await res.json().catch(() => ({}));
        console.error("Pipeline execution failed:", res.status, errorData);
        throw new Error(
          errorData?.detail || `Failed to execute pipeline: ${res.statusText}`
        );
      }

      console.log("Pipeline submitted successfully.");
      triggerPipelinesRefetch();

      options?.onSuccess?.();
      return true;
    } catch (error) {
      console.error("Error executing pipeline:", error);
      const errorMessage =
        error instanceof Error
          ? error.message
          : t("pipeline_execution.execution_error");
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
            to_chain_caip2: chainIdToCaip2(options?.chainId),
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
        showToast(`${t("pipeline_execution.buy_order_placed")}`, "success");
        options?.onSuccess?.();
      },
      onError: (error) => {
        showToast(`${t("pipeline_execution.failed_to_buy_token")}`, "error");
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
    let adjustedDecimals = tokenDecimals;
    if (tokenName === "USDC") {
      adjustedDecimals = 6;
    }
    const rawAmount = Math.floor(
      tokenAmount * 10 ** adjustedDecimals
    ).toString();

    const sellPipeline: Pipeline = {
      steps: [
        {
          action: {
            type: PipelineActionType.SwapOrder,
            input_token: tokenAddress,
            output_token: "So11111111111111111111111111111111111111112", // wSOL
            amount: rawAmount,
            from_chain_caip2: chainIdToCaip2(options?.chainId),
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

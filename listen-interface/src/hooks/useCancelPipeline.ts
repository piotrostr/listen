import { usePrivy } from "@privy-io/react-auth";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { config } from "../config";
import { useToast } from "../contexts/ToastContext";
import { usePrivyWallets } from "./usePrivyWallet";

interface CancelOptions {
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export function useCancelPipeline() {
  const [isCancelling, setIsCancelling] = useState(false);
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

  const invalidatePipelines = () => {
    queryClient.refetchQueries({
      queryKey: ["pipelines"],
    });
  };

  const cancelPipeline = async (
    pipelineId: string,
    options?: CancelOptions
  ) => {
    setIsCancelling(true);
    try {
      const token = await getAccessToken();
      const res = await fetch(
        `${config.engineEndpoint}/pipeline/${pipelineId}/cancel`,
        {
          method: "POST",
          headers: {
            Authorization: `Bearer ${token}`,
            "Content-Type": "application/json",
          },
        }
      );

      if (!res.ok) {
        throw new Error("Failed to cancel pipeline");
      }

      invalidatePipelines();
      invalidateSolanaPortfolio();

      showToast(t("pipeline_execution.pipeline_cancelled"), "success");
      options?.onSuccess?.();
      return true;
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : t("pipeline_execution.cancel_error");
      showToast(errorMessage, "error");
      options?.onError?.(
        error instanceof Error ? error : new Error(errorMessage)
      );
      return false;
    } finally {
      setIsCancelling(false);
    }
  };

  const cancelStep = async (
    pipelineId: string,
    stepId: string,
    options?: CancelOptions
  ) => {
    setIsCancelling(true);
    try {
      const token = await getAccessToken();
      const res = await fetch(
        `${config.engineEndpoint}/pipeline/${pipelineId}/step/${stepId}/cancel`,
        {
          method: "POST",
          headers: {
            Authorization: `Bearer ${token}`,
            "Content-Type": "application/json",
          },
        }
      );

      if (!res.ok) {
        throw new Error("Failed to cancel step");
      }

      invalidatePipelines();
      invalidateSolanaPortfolio();

      showToast(t("pipelines.step_cancelled"), "success");
      options?.onSuccess?.();
      return true;
    } catch (error) {
      const errorMessage =
        error instanceof Error
          ? error.message
          : t("pipelines.cancel_step_error");
      showToast(errorMessage, "error");
      options?.onError?.(
        error instanceof Error ? error : new Error(errorMessage)
      );
      return false;
    } finally {
      setIsCancelling(false);
    }
  };

  return {
    isCancelling,
    cancelPipeline,
    cancelStep,
  };
}

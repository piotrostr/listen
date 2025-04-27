import { useState } from "react";
import { useTranslation } from "react-i18next";
import { MdCancel } from "react-icons/md";
import { useCancelPipeline } from "../hooks/useCancelPipeline";
import { ExtendedPipeline, ExtendedPipelineCondition } from "../types/api";
import {
  PipelineActionType,
  PipelineCondition,
  PipelineConditionType,
} from "../types/pipeline";
import { CopyIcon } from "./CopyIcon";
import { NotificationPipelineStep } from "./NotificationPipelineStep";
import { SwapPipelineStep } from "./SwapPipelineStep";

interface ExtendedPipelineProps {
  pipeline: ExtendedPipeline;
}

export function ExtendedPipelineDisplay({ pipeline }: ExtendedPipelineProps) {
  const { t } = useTranslation();
  const { cancelPipeline, cancelStep, isCancelling } = useCancelPipeline();

  // Convert steps object to array and sort by execution order
  const orderedSteps = Object.entries(pipeline.steps).map(([index, step]) => ({
    ...step,
    index: parseInt(index),
  }));

  // Check if pipeline is in pending state - show cancel button only if any step is pending
  const isPipelinePending = Object.values(pipeline.steps).some(
    (step) => step.status === "Pending"
  );

  // Check if pipeline has multiple steps
  const hasMultipleSteps = Object.keys(pipeline.steps).length > 1;

  const renderCondition = (
    condition: ExtendedPipelineCondition
  ): PipelineCondition => {
    const condType = condition.condition_type;

    if ("PriceBelow" in condType) {
      return {
        type: PipelineConditionType.PriceBelow,
        value: condType.PriceBelow.value,
        asset: condType.PriceBelow.asset,
      };
    }
    if ("PriceAbove" in condType) {
      return {
        type: PipelineConditionType.PriceAbove,
        value: condType.PriceAbove.value,
        asset: condType.PriceAbove.asset,
      };
    }
    if ("Now" in condType) {
      return {
        type: PipelineConditionType.Now,
        asset: condType.Now.asset,
        value: 0,
      };
    }

    throw new Error(`Unknown condition type: ${JSON.stringify(condType)}`);
  };

  const handleCancelPipeline = () => {
    cancelPipeline(pipeline.id);
  };

  const handleCancelStep = (stepId: string) => {
    cancelStep(pipeline.id, stepId);
  };

  // Format creation date to be more human-readable
  const formatCreationDate = (dateString: string): string => {
    const date = new Date(dateString);
    const now = new Date();
    const diffInMs = now.getTime() - date.getTime();
    const diffInMinutes = Math.floor(diffInMs / (1000 * 60));
    const diffInHours = Math.floor(diffInMs / (1000 * 60 * 60));
    const diffInDays = Math.floor(diffInMs / (1000 * 60 * 60 * 24));

    if (diffInMinutes < 1) return "Just now";
    if (diffInMinutes < 60) return `${diffInMinutes}m ago`;
    if (diffInHours < 24) return `${diffInHours}h ago`;
    if (diffInDays < 7) return `${diffInDays}d ago`;

    return date.toLocaleDateString();
  };

  // Format ID to be shorter and more readable
  const formatId = (id: string): string => {
    if (id.length <= 12) return id;
    return `${id.substring(0, 6)}...${id.substring(id.length - 4)}`;
  };

  const [isCopied, setIsCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(pipeline.id);
    setIsCopied(true);
    setTimeout(() => setIsCopied(false), 2000);
  };

  return (
    <>
      <div className="flex flex-row items-center gap-3">
        <div className="text-white text-xs sm:text-sm flex items-center gap-1">
          <span className="font-bold">{t("pipelines.id")}:</span>{" "}
          <span className="text-gray-400" title={pipeline.id}>
            {formatId(pipeline.id)}
          </span>
          <button
            className="text-gray-400 hover:text-blue-400 transition-colors"
            onClick={handleCopy}
            title="Copy full ID"
          >
            {isCopied ? "✅" : <CopyIcon />}
          </button>
        </div>
        <div className="flex justify-between items-center">
          <div className="text-white text-xs sm:text-sm">
            <span
              className="text-gray-400"
              title={new Date(pipeline.created_at).toLocaleString()}
            >
              ({formatCreationDate(pipeline.created_at)})
            </span>
          </div>
          {isPipelinePending && (
            <CancelButton
              onClick={handleCancelPipeline}
              disabled={isCancelling}
            />
          )}
        </div>
      </div>

      {/* Steps */}
      <div className="space-y-3 mt-4">
        {orderedSteps.map((step) => {
          // Only show cancel button for steps that can be cancelled (not completed or failed)
          // And only when there are multiple steps in the pipeline
          const canCancel = step.status === "Pending" && hasMultipleSteps;

          if ("Order" in step.action) {
            return (
              <div key={step.id} className="flex items-start justify-between">
                <div className="flex-grow">
                  <SwapPipelineStep
                    key={step.id}
                    step={{
                      action: {
                        type: PipelineActionType.SwapOrder,
                        amount: step.action.Order.amount,
                        from_chain_caip2: step.action.Order.from_chain_caip2,
                        input_token: step.action.Order.input_token,
                        output_token: step.action.Order.output_token,
                        to_chain_caip2: step.action.Order.to_chain_caip2,
                      },
                      conditions: step.conditions.map((condition) =>
                        renderCondition(condition)
                      ),
                    }}
                    status={step.status}
                    transactionHash={step.transaction_hash}
                    error={step.error ?? null}
                    compact={true}
                  />
                </div>
                {canCancel && (
                  <CancelButton
                    onClick={() => handleCancelStep(step.id)}
                    disabled={isCancelling}
                  />
                )}
              </div>
            );
          } else if ("Notification" in step.action) {
            return (
              <div key={step.id} className="flex items-start justify-between">
                <div className="flex-grow">
                  <NotificationPipelineStep
                    key={step.id}
                    step={{
                      action: {
                        type: PipelineActionType.Notification,
                        input_token: step.action.Notification.input_token,
                        message: step.action.Notification.message,
                      },
                      conditions: step.conditions.map((condition) =>
                        renderCondition(condition)
                      ),
                    }}
                    status={step.status}
                  />
                </div>
                {canCancel && (
                  <CancelButton
                    onClick={() => handleCancelStep(step.id)}
                    disabled={isCancelling}
                  />
                )}
              </div>
            );
          }
          return null;
        })}
      </div>
    </>
  );
}

const CancelButton = ({
  onClick,
  disabled,
}: {
  onClick: () => void;
  disabled: boolean;
}) => {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className="p-1 ml-2 rounded text-red-500 self-start mt-2"
    >
      <MdCancel size={16} />
    </button>
  );
};

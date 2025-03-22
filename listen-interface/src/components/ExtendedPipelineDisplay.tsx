import { useTranslation } from "react-i18next";
import { MdCancel } from "react-icons/md";
import { useCancelPipeline } from "../hooks/useCancelPipeline";
import { ExtendedPipeline, ExtendedPipelineCondition } from "../types/api";
import {
  PipelineActionType,
  PipelineCondition,
  PipelineConditionType,
} from "../types/pipeline";
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

  return (
    <>
      {/* Header */}
      <div className="flex justify-between">
        <div className="flex gap-2 flex-col">
          <div className="text-white text-xs sm:text-sm">
            <span className="font-bold">{t("pipelines.id")}:</span>{" "}
            <span className="text-gray-400">{pipeline.id}</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <div className="text-white text-xs sm:text-sm">
            <span className="font-bold">{t("pipelines.created")}:</span>{" "}
            <span className="text-gray-400">
              {new Date(pipeline.created_at).toLocaleString()}
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

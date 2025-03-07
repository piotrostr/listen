import { useTranslation } from "react-i18next";
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

  // Convert steps object to array and sort by execution order
  const orderedSteps = Object.entries(pipeline.steps).map(([index, step]) => ({
    ...step,
    index: parseInt(index),
  }));

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
  return (
    <>
      {/* Header */}
      <div className="flex justify-between flex-col">
        <div className="flex gap-2 flex-col">
          <div className="text-purple-300 text-xs sm:text-sm">
            <span className="font-bold">{t("pipelines.id")}:</span>{" "}
            {pipeline.id}
          </div>
        </div>
        <div className="text-purple-300 text-xs sm:text-sm">
          <span className="font-bold">{t("pipelines.created")}:</span>{" "}
          {new Date(pipeline.created_at).toLocaleString()}
        </div>
      </div>

      {/* Steps */}
      <div className="space-y-3 mt-4">
        {orderedSteps.map((step) => {
          if ("Order" in step.action) {
            return (
              <div key={step.id}>
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
            );
          } else if ("Notification" in step.action) {
            return (
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
              />
            );
          }
          return null;
        })}
      </div>
    </>
  );
}

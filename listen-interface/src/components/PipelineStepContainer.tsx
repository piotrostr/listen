import { PipelineCondition, PipelineConditionType } from "../types/pipeline";

interface PipelineStepContainerProps {
  index: number;
  children: React.ReactNode;
  conditions: PipelineCondition[];
}

export const PipelineStepContainer = ({
  index,
  children,
  conditions,
}: PipelineStepContainerProps) => {
  return (
    <div className="border border-purple-500/30 rounded-lg p-4 bg-black/40 backdrop-blur-sm">
      <div className="flex items-center gap-4">
        <div className="text-sm text-purple-300">{index + 1}</div>
        {children}
      </div>

      {/* Conditions */}
      {conditions.length > 0 && (
        <div className="mt-3 pt-3 border-t border-purple-500/30">
          <div className="text-sm text-purple-300">Conditions:</div>
          {conditions.map((condition, index) => (
            <div key={index} className="mt-1 text-sm text-purple-200">
              {condition.type === PipelineConditionType.Now
                ? "Execute immediately"
                : condition.type === PipelineConditionType.PriceAbove
                  ? `Price above ${condition.value} for ${condition.asset}`
                  : `Price below ${condition.value} for ${condition.asset}`}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

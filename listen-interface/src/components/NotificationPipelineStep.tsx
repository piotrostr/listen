import { useTranslation } from "react-i18next";
import { useSolanaToken } from "../hooks/useToken";
import { PipelineActionType, PipelineStep } from "../types/pipeline";
import { PipelineStepContainer } from "./PipelineStepContainer";

interface NotificationPipelineStepProps {
  step: PipelineStep;
}

export const NotificationPipelineStep = ({
  step,
}: NotificationPipelineStepProps) => {
  const { t } = useTranslation();

  if (step.action.type !== PipelineActionType.Notification) {
    return null;
  }

  const inputToken = step.action.input_token
    ? useSolanaToken(step.action.input_token)
    : null;

  const tokenImage = inputToken?.data?.logoURI;
  const tokenName = inputToken?.data?.symbol;

  return (
    <PipelineStepContainer
      conditions={step.conditions ?? []}
      transactionHash={null}
      error={null}
    >
      <div className="flex-1">
        <div className="flex items-center gap-3">
          {tokenImage && (
            <img
              src={tokenImage}
              alt={tokenName}
              className="w-8 h-8 rounded-full"
            />
          )}
          <div>
            <div className="font-bold text-white">
              {t("pipelines.send_notification")}
            </div>
            <div className="text-gray-400">{step.action.message}</div>
          </div>
        </div>
      </div>
    </PipelineStepContainer>
  );
};

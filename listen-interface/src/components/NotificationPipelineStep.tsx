import { useSolanaToken } from "../hooks/useToken";
import { PipelineStep } from "../types/pipeline";
import { PipelineStepContainer } from "./PipelineStepContainer";
import { Spinner } from "./Spinner";

interface NotificationPipelineStepProps {
  index: number;
  step: PipelineStep;
}

export const NotificationPipelineStep = ({
  index,
  step,
}: NotificationPipelineStepProps) => {
  const inputToken = useSolanaToken(step.action.input_token);
  if (!inputToken.data) return <Spinner />;

  const tokenImage = inputToken.data?.logoURI;
  const tokenName = inputToken.data?.symbol;

  return (
    <PipelineStepContainer index={index} conditions={step.conditions}>
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
            <div className="font-bold text-purple-100">Send a notification</div>
          </div>
        </div>
      </div>
    </PipelineStepContainer>
  );
};

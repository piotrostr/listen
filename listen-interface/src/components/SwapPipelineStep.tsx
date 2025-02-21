import { useToken } from "../hooks/useToken";
import { caip2ToChainId, formatAmount } from "../hooks/util";
import { PipelineActionType, PipelineStep } from "../types/pipeline";
import { PipelineStepContainer } from "./PipelineStepContainer";
import { Spinner } from "./Spinner";
import { SwapToken } from "./SwapToken";

interface SwapPipelineStepProps {
  index: number;
  step: PipelineStep;
  status?: "Pending" | "Completed" | "Failed" | "Cancelled";
  transactionHash: string | null;
  error: string | null;
}

export const SwapPipelineStep = ({
  index,
  step,
  status,
  transactionHash,
  error,
}: SwapPipelineStepProps) => {
  if (step.action.type !== PipelineActionType.SwapOrder) {
    throw new Error("SwapPipelineStep received non-swap action type");
  }

  const { data: inputToken, isLoading: inputTokenLoading } = useToken(
    step.action.input_token,
    step.action.from_chain_caip2
  );

  const { data: outputToken, isLoading: outputTokenLoading } = useToken(
    step.action.output_token,
    step.action.to_chain_caip2
  );

  const inputImage = inputToken?.logoURI;
  const outputImage = outputToken?.logoURI;
  const inputName = inputToken?.symbol;
  const outputName = outputToken?.symbol;

  const fromChain = caip2ToChainId(step.action.from_chain_caip2);
  const toChain = caip2ToChainId(step.action.to_chain_caip2);

  return (
    <PipelineStepContainer
      index={index}
      conditions={step.conditions}
      status={status}
      transactionHash={transactionHash}
      error={error}
    >
      <div className="flex-1">
        {inputTokenLoading ? (
          <Spinner />
        ) : (
          <SwapToken
            image={inputImage}
            name={inputName}
            amount={
              inputToken
                ? formatAmount(step.action.amount, inputToken.decimals)
                : step.action.amount
            }
            chainId={fromChain}
            address={inputToken?.address}
            showAmount={true}
          />
        )}
      </div>

      {/* Arrow */}
      <div className="text-purple-500">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
          className="w-6 h-6"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3"
          />
        </svg>
      </div>

      <div className="flex-1">
        {outputTokenLoading ? (
          <Spinner />
        ) : (
          <SwapToken
            image={outputImage}
            name={outputName}
            chainId={toChain}
            address={outputToken?.address}
          />
        )}
      </div>
    </PipelineStepContainer>
  );
};

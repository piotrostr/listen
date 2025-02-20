import { useSolanaToken } from "../hooks/useToken";
import { caip2ToChainId, formatAmount } from "../hooks/util";
import { PipelineActionType, PipelineStep } from "../types/pipeline";
import { PipelineStepContainer } from "./PipelineStepContainer";

interface SwapPipelineStepProps {
  index: number;
  step: PipelineStep;
}

export const SwapPipelineStep = ({ index, step }: SwapPipelineStepProps) => {
  if (step.action.type !== PipelineActionType.SwapOrder) {
    throw new Error("SwapPipelineStep received non-swap action type");
  }

  const inputToken = useSolanaToken(step.action.input_token);
  const outputToken = useSolanaToken(step.action.output_token);

  const inputImage = inputToken.data?.logoURI;
  const outputImage = outputToken.data?.logoURI;
  const inputName = inputToken.data?.symbol;
  const outputName = outputToken.data?.symbol;

  const fromChain = caip2ToChainId(step.action.from_chain_caip2);
  const toChain = caip2ToChainId(step.action.to_chain_caip2);

  return (
    <PipelineStepContainer index={index} conditions={step.conditions}>
      {/* Input Token */}
      <div className="flex-1">
        <div className="flex items-center gap-3">
          <div className="flex items-center">
            {fromChain && (
              <img
                src={`https://dd.dexscreener.com/ds-data/chains/${fromChain.toLowerCase()}.png`}
                alt={fromChain}
                className="w-4 h-4 rounded-full mr-2"
              />
            )}
            {inputImage && (
              <img
                src={inputImage}
                alt={inputName}
                className="w-8 h-8 rounded-full"
              />
            )}
          </div>
          <div>
            <div className="font-bold text-purple-100">{inputName}</div>
            <div className="text-sm text-purple-300">
              Amount:{" "}
              {inputToken.data
                ? formatAmount(step.action.amount, inputToken.data.decimals)
                : step.action.amount}
            </div>
            <div className="text-sm text-gray-400">
              {inputToken.data?.address.slice(0, 4)}...
              {inputToken.data?.address.slice(-4)}
            </div>
          </div>
        </div>
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

      {/* Output Token */}
      <div className="flex-1">
        <div className="flex items-center gap-3">
          <div className="flex items-center">
            {toChain && (
              <img
                src={`https://dd.dexscreener.com/ds-data/chains/${toChain.toLowerCase()}.png`}
                alt={toChain}
                className="w-5 h-5 rounded-full mr-2"
              />
            )}
            {outputImage && (
              <img
                src={outputImage}
                alt={outputName}
                className="w-8 h-8 rounded-full"
              />
            )}
          </div>
          <div>
            <div className="font-bold text-purple-100">{outputName}</div>
            <div className="text-sm text-purple-300">
              Amount:{" "}
              {outputToken.data
                ? formatAmount(step.action.amount, outputToken.data.decimals)
                : step.action.amount}
            </div>
            <div className="text-sm text-gray-400">
              {outputToken.data?.address.slice(0, 4)}...
              {outputToken.data?.address.slice(-4)}
            </div>
          </div>
        </div>
      </div>
    </PipelineStepContainer>
  );
};

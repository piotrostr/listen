import { usePrivy } from "@privy-io/react-auth";
import { useState } from "react";
import { config } from "../config";
import { useToast } from "../contexts/ToastContext";
import { useSolanaToken } from "../hooks/useToken";
import {
  Pipeline,
  PipelineActionType,
  PipelineCondition,
  PipelineConditionType,
  PipelineSchema,
  PipelineStep,
} from "../types/pipeline";
import { Spinner } from "./Spinner";

interface PipelineProps {
  pipeline: Pipeline;
}

interface SwapPipelineStepProps {
  index: number;
  step: PipelineStep;
}

interface PipelineStepContainerProps {
  index: number;
  children: React.ReactNode;
  conditions: PipelineCondition[];
}

const PipelineStepContainer = ({
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
              {condition.type === "PriceAbove" ? "Price above" : "Price below"}{" "}
              {condition.value} for {condition.asset}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export const SwapPipelineStep = ({ index, step }: SwapPipelineStepProps) => {
  if (step.action.type !== PipelineActionType.SwapOrder) {
    throw new Error("SwapPipelineStep received non-swap action type");
  }

  console.log(step);

  const inputToken = useSolanaToken(step.action.input_token);
  const outputToken = useSolanaToken(step.action.output_token);

  const inputImage = inputToken.data?.logoURI;
  const outputImage = outputToken.data?.logoURI;
  const inputName = inputToken.data?.symbol;
  const outputName = outputToken.data?.symbol;

  const formatAmount = (amount: string, decimals: number) => {
    const amountNum = parseFloat(amount);
    return (amountNum / Math.pow(10, decimals)).toString();
  };

  return (
    <PipelineStepContainer index={index} conditions={step.conditions}>
      {/* Input Token */}
      <div className="flex-1">
        <div className="flex items-center gap-3">
          {inputImage && (
            <img
              src={inputImage}
              alt={inputName}
              className="w-8 h-8 rounded-full"
            />
          )}
          <div>
            <div className="font-bold text-purple-100">{inputName}</div>
            <div className="text-sm text-purple-300">
              Amount:{" "}
              {inputToken.data
                ? formatAmount(step.action.amount, inputToken.data.decimals)
                : step.action.amount}
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
          {outputImage && (
            <img
              src={outputImage}
              alt={outputName}
              className="w-8 h-8 rounded-full"
            />
          )}
          <div className="font-bold text-purple-100">{outputName}</div>
        </div>
      </div>
    </PipelineStepContainer>
  );
};

export const NotificationPipelineStep = ({
  index,
  step,
}: SwapPipelineStepProps) => {
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

export function PipelineDisplay({ pipeline }: PipelineProps) {
  const { getAccessToken } = usePrivy();
  const [status, setStatus] = useState<
    "loading" | "pending" | "approved" | "rejected"
  >("pending");
  const { showToast } = useToast();

  const sendPipelineForExecution = async () => {
    setStatus("loading");
    try {
      const token = await getAccessToken();
      const res = await fetch(config.API_BASE_URL + "/v1/engine/pipeline", {
        method: "POST",
        body: JSON.stringify(pipeline),
        headers: {
          Authorization: `Bearer ${token}`,
          "Content-Type": "application/json",
        },
      });

      if (!res.ok) {
        throw new Error("Failed to send pipeline for execution");
      }

      const data = await res.json();
      console.log(data);

      showToast("Pipeline scheduled for execution", "success");
      setStatus("approved");
    } catch (error) {
      showToast(
        error instanceof Error ? error.message : "An error occurred",
        "error"
      );
      setStatus("pending");
    }
  };

  return (
    <div className="space-y-4">
      {pipeline.steps.map((step, index) => {
        switch (step.action.type) {
          case PipelineActionType.SwapOrder:
            return (
              <SwapPipelineStep
                key={`swap-${index}`}
                index={index}
                step={step}
              />
            );
          case PipelineActionType.Notification:
            return (
              <NotificationPipelineStep
                key={`notification-${index}`}
                index={index}
                step={step}
              />
            );
          default:
            return null;
        }
      })}
      {status === "loading" ? (
        <Spinner />
      ) : (
        <PipelineMenu
          status={status}
          setStatus={setStatus}
          sendPipelineForExecution={sendPipelineForExecution}
        />
      )}
    </div>
  );
}

function PipelineMenu({
  status,
  setStatus,
  sendPipelineForExecution,
}: {
  status: "pending" | "approved" | "rejected";
  setStatus: (status: "pending" | "approved" | "rejected") => void;
  sendPipelineForExecution: () => void;
}) {
  const Container = ({ children }: { children: React.ReactNode }) => {
    return <div className="flex gap-2">{children}</div>;
  };

  switch (status) {
    case "pending":
      return (
        <Container>
          <>
            <button
              onClick={sendPipelineForExecution}
              className="px-4 py-2 bg-green-500/20 hover:bg-green-500/30 text-green-300 border border-green-500/30 rounded-lg transition-colors"
            >
              Approve
            </button>
            <button
              onClick={() => setStatus("rejected")}
              className="px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-300 border border-red-500/30 rounded-lg transition-colors"
            >
              Reject
            </button>
          </>
        </Container>
      );
    case "approved":
      return (
        <Container>
          <div className="text-green-400 flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              fill="currentColor"
              className="w-5 h-5"
            >
              <path
                fillRule="evenodd"
                d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm13.36-1.814a.75.75 0 10-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z"
                clipRule="evenodd"
              />
            </svg>
            <span>Pipeline scheduled for execution</span>
          </div>
        </Container>
      );
    case "rejected":
      return (
        <Container>
          <div className="text-red-400 flex items-center gap-2">
            <span>Pipeline rejected</span>
          </div>
        </Container>
      );
  }
}

export function serializePipeline(pipeline: Pipeline): string {
  return JSON.stringify(pipeline);
}

export function deserializePipeline(serialized: string): Pipeline {
  const parsed = JSON.parse(serialized);
  return PipelineSchema.parse(parsed);
}

export const mockOrderPipeline: Pipeline = {
  steps: [
    {
      action: {
        type: PipelineActionType.SwapOrder,
        input_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        output_token: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        amount: "1000000000000000000",
        from_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        to_chain_caip2: "eip155:1",
      },
      conditions: [],
    },
    {
      action: {
        type: PipelineActionType.SwapOrder,
        input_token: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        output_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        amount: "1000000000000000000",
        from_chain_caip2: "eip155:1",
        to_chain_caip2: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
      },
      conditions: [
        {
          type: PipelineConditionType.PriceAbove,
          asset: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
          value: 0.052,
        },
      ],
    },
  ],
};

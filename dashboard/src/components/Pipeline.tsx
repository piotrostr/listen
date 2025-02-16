import { useState } from "react";
import { useSolanaTokens } from "../hooks/useToken";
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
  const { data: tokens } = useSolanaTokens([
    step.action.input_token,
    step.action.output_token,
  ]);
  if (!tokens) return <Spinner />;

  const inputImage = tokens[step.action.input_token]?.mpl.ipfs_metadata?.image;
  const outputImage =
    tokens[step.action.output_token]?.mpl.ipfs_metadata?.image;
  const inputName = tokens[step.action.input_token]?.mpl?.name;
  const outputName = tokens[step.action.output_token]?.mpl?.name;

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
            {step.action.percentage ? (
              <div className="text-sm text-purple-300">
                Percentage: {step.action.percentage * 100}%
              </div>
            ) : (
              <div className="text-sm text-purple-300">
                Amount: {step.action.amount}
              </div>
            )}
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
  const { data: tokens } = useSolanaTokens([step.action.input_token]);
  if (!tokens) return <Spinner />;

  const tokenImage = tokens[step.action.input_token]?.mpl.ipfs_metadata?.image;
  const tokenName = tokens[step.action.input_token]?.mpl?.name;

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
            <div className="font-bold text-purple-100">
              Notify when conditions are met for {tokenName}
            </div>
          </div>
        </div>
      </div>
    </PipelineStepContainer>
  );
};

export function PipelineDisplay({ pipeline }: PipelineProps) {
  const [status, setStatus] = useState<
    "loading" | "pending" | "approved" | "rejected"
  >("pending");

  const sendPipelineForExecution = () => {
    // TODO: Implement pipeline execution
    setStatus("loading");
    setTimeout(() => {
      setStatus("approved");
    }, 1000);
  };

  if (status === "rejected") {
    return null;
  }

  return (
    <div className="space-y-4">
      <h1 className="text-xl font-bold text-purple-100 mb-4">Pipeline</h1>
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
  return (
    <div className="flex gap-2">
      {status === "pending" ? (
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
      ) : status === "approved" ? (
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
      ) : null}
    </div>
  );
}

export interface Pipeline {
  steps: PipelineStep[];
}

interface PipelineStep {
  action: PipelineAction;
  conditions: PipelineCondition[];
}

enum PipelineActionType {
  SwapOrder = "SwapOrder",
  Notification = "Notification",
}

interface PipelineAction {
  type: PipelineActionType;
  input_token: string;
  output_token: string;
  // message in case of Notification
  message: string | null;
  // one of amount/percentage for SwapOrder
  amount: number | null;
  percentage: number | null;
}

enum PipelineConditionType {
  PriceAbove = "PriceAbove",
  PriceBelow = "PriceBelow",
  Now = "Now",
}

interface PipelineCondition {
  type: PipelineConditionType;
  asset: string;
  value: number;
}

export function serializePipeline(pipeline: Pipeline): string {
  return JSON.stringify(pipeline);
}

export function deserializePipeline(serialized: string): Pipeline {
  const parsed = JSON.parse(serialized);

  if (!Array.isArray(parsed.steps)) {
    throw new Error("Invalid pipeline format: steps must be an array");
  }

  parsed.steps.forEach((step: any) => {
    if (!Object.values(PipelineActionType).includes(step.action.type)) {
      throw new Error(`Invalid action type: ${step.action.type}`);
    }

    if (!Array.isArray(step.conditions)) {
      throw new Error("Invalid step format: conditions must be an array");
    }

    step.conditions.forEach((condition: any) => {
      if (!Object.values(PipelineConditionType).includes(condition.type)) {
        throw new Error(`Invalid condition type: ${condition.type}`);
      }
    });
  });

  return parsed as Pipeline;
}

export const mockOrderPipeline: Pipeline = {
  steps: [
    {
      action: {
        type: PipelineActionType.SwapOrder,
        input_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        output_token: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        amount: 1000,
        percentage: null,
        message: null,
      },
      conditions: [],
    },
    {
      action: {
        type: PipelineActionType.SwapOrder,
        input_token: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
        output_token: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        amount: null,
        percentage: 0.5,
        message: null,
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

import { useState } from "react";
import { useEoaExecution } from "../hooks/useEoaExecution";
import { useAllWallets } from "../hooks/useAllWallets";
import { Pipeline, PipelineActionType } from "../types/pipeline";
import { NotificationPipelineStep } from "./NotificationPipelineStep";
import { PipelineMenu } from "./PipelineMenu";
import { Spinner } from "./Spinner";
import { SwapPipelineStep } from "./SwapPipelineStep";

interface PipelineProps {
  pipeline: Pipeline;
}

export function PipelineDisplay({ pipeline }: PipelineProps) {
  const [status, setStatus] = useState<
    "loading" | "pending" | "approved" | "rejected"
  >("pending");
  const { handleEoaSolana, handleEoaEvm } = useEoaExecution();
  const { evmAddress, solanaAddress } = useAllWallets();

  const executeFromEoa = async () => {
    setStatus("loading");

    for (const step of pipeline.steps) {
      switch (step.action.type) {
        case PipelineActionType.SwapOrder:
          const action = step.action;

          // Solana swap
          if (
            action.from_chain_caip2?.startsWith("solana:") &&
            action.to_chain_caip2?.startsWith("solana:") &&
            solanaAddress
          ) {
            const result = await handleEoaSolana(action, solanaAddress);
            if (!result) {
              setStatus("pending");
              return;
            }
            setStatus("approved");
          }

          // EVM swap
          if (
            step.action.from_chain_caip2?.startsWith("eip155:") &&
            step.action.to_chain_caip2?.startsWith("eip155:") &&
            evmAddress
          ) {
            const result = await handleEoaEvm(action, evmAddress);
            if (!result) {
              setStatus("pending");
              return;
            }
            setStatus("approved");
          }
      }
    }

    setStatus("pending");
  };

  return (
    <div className="space-y-4">
      {pipeline.steps.map((step, index) => {
        switch (step.action.type) {
          case PipelineActionType.SwapOrder:
            return (
              <SwapPipelineStep
                key={`swap-${index}`}
                step={step}
                transactionHash={null}
                error={null}
              />
            );
          case PipelineActionType.Notification:
            return (
              <NotificationPipelineStep
                key={`notification-${index}`}
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
          executeFromEoa={() => executeFromEoa()}
        />
      )}
    </div>
  );
}

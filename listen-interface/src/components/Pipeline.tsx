import { useState } from "react";
import { useEoaExecution } from "../hooks/useEoaExecution";
import { usePipelineExecution } from "../hooks/usePipelineExecution";
import { useWalletStore } from "../store/walletStore";
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
  const { isExecuting, executePipeline } = usePipelineExecution();
  const { handleEoaSolana, handleEoaEvm } = useEoaExecution();

  const sendPipelineForExecution = async () => {
    setStatus("loading");
    const success = await executePipeline(pipeline, {
      onSuccess: () => setStatus("approved"),
      onError: () => setStatus("pending"),
    });
    if (!success) {
      setStatus("pending");
    }
  };

  const { activeWallet, eoaEvmAddress, eoaSolanaAddress } = useWalletStore();

  const executeFromEoa = async () => {
    setStatus("loading");
    if (!eoaEvmAddress) {
      setStatus("pending");
      return;
    }
    for (const step of pipeline.steps) {
      switch (step.action.type) {
        case PipelineActionType.SwapOrder:
          const action = step.action;
          if (
            action.from_chain_caip2?.startsWith("solana:") &&
            action.to_chain_caip2?.startsWith("solana:") &&
            eoaSolanaAddress
          ) {
            const result = await handleEoaSolana(action, eoaSolanaAddress);
            if (!result) {
              setStatus("pending");
              return;
            }
            setStatus("approved");
          }

          if (
            step.action.from_chain_caip2?.startsWith("eip155:") &&
            step.action.to_chain_caip2?.startsWith("eip155:") &&
            eoaEvmAddress
          ) {
            const result = await handleEoaEvm(action, eoaEvmAddress);
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
      {isExecuting || status === "loading" ? (
        <Spinner />
      ) : (
        <PipelineMenu
          status={status}
          setStatus={setStatus}
          sendPipelineForExecution={
            activeWallet === "listen" ? sendPipelineForExecution : undefined
          }
          executeFromEoa={
            activeWallet !== "listen" ? () => executeFromEoa() : undefined
          }
        />
      )}
    </div>
  );
}

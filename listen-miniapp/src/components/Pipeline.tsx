import { useState } from "react";
import { worldchainEnabled } from "../config/env";
import { useEoaExecution } from "../hooks/useEoaExecution";
import { usePipelineExecution } from "../hooks/usePipelineExecution";
import { useWorldAuth } from "../hooks/useWorldLogin";
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
  const { handleEoaSolana, handleEoaEvm, handleEoaWorld } = useEoaExecution();

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

  const { worldUserAddress: worldchainAddress } = useWorldAuth();

  const executeFromEoa = async () => {
    if (!eoaEvmAddress && !eoaSolanaAddress && !worldchainAddress) {
      throw new Error("No EOA address found");
    }
    setStatus("loading");
    for (const step of pipeline.steps) {
      switch (step.action.type) {
        case PipelineActionType.SwapOrder:
          const action = step.action;
          if (worldchainEnabled) {
            if (!action.from_chain_caip2 || !action.to_chain_caip2) {
              throw new Error("Missing chain CAIP2");
            }
            if (
              action.from_chain_caip2 !== "eip155:480" ||
              action.to_chain_caip2 !== "eip155:480"
            ) {
              throw new Error("Invalid chain CAIP2 for Worldchain");
            }
            if (!worldchainAddress) {
              throw new Error("Missing Worldchain address");
            }
            const result = await handleEoaWorld(action, worldchainAddress);
            if (!result) {
              setStatus("pending");
              return;
            }
            setStatus("approved");
          }
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
        <>
          {worldchainEnabled ? (
            <PipelineMenu
              status={status}
              setStatus={setStatus}
              executeFromEoa={executeFromEoa}
            />
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
        </>
      )}
    </div>
  );
}

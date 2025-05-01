import { useSolanaWallets, useWallets } from "@privy-io/react-auth";
import { Connection, VersionedTransaction } from "@solana/web3.js";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { type EIP1193Provider } from "viem";
import { ensureApprovals } from "../approvals";
import { swapStepToTransaction } from "../eoa-tx";
import { usePipelineExecution } from "../hooks/usePipelineExecution";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";
import {
  Pipeline,
  PipelineActionType,
  SwapOrderAction,
} from "../types/pipeline";
import { NotificationPipelineStep } from "./NotificationPipelineStep";
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
  const { wallets: evmWallets } = useWallets();
  const { wallets: solanaWallets } = useSolanaWallets();
  const { refreshPortfolio } = usePortfolioStore();

  const handleEoaSolana = async (
    action: SwapOrderAction,
    eoaSolanaAddress: string
  ): Promise<boolean> => {
    try {
      const wallet = solanaWallets.find((w) => w.address === eoaSolanaAddress);
      if (wallet) {
        const tx = await swapStepToTransaction(action, eoaSolanaAddress);
        const transaction = VersionedTransaction.deserialize(
          Uint8Array.from(Buffer.from(tx?.data ?? "", "base64"))
        );
        const connection = new Connection(
          import.meta.env.VITE_SOLANA_RPC_URL ||
            "https://api.mainnet-beta.solana.com"
        );
        await wallet.sendTransaction(transaction, connection);
        refreshPortfolio();
        return true;
      }
    } catch (error) {
      console.error(error);
      return false;
    }
    return false;
  };

  const handleEoaEvm = async (
    action: SwapOrderAction,
    eoaEvmAddress: string
  ): Promise<boolean> => {
    try {
      const wallet = evmWallets.find((w) => w.address === eoaEvmAddress);
      if (wallet) {
        const provider = await wallet.getEthereumProvider();
        const approvalsTx = await ensureApprovals(
          action,
          eoaEvmAddress,
          provider as unknown as EIP1193Provider
        );
        if (approvalsTx) {
          await provider.request({
            method: "eth_sendTransaction",
            params: [approvalsTx],
          });
        }
        const tx = await swapStepToTransaction(action, eoaEvmAddress);
        await provider.request({
          method: "eth_sendTransaction",
          params: [tx],
        });
        setStatus("approved");
        return true;
      }
    } catch (error) {
      console.error(error);
      setStatus("pending");
    }
    return false;
  };

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

function PipelineMenu({
  status,
  setStatus,
  sendPipelineForExecution,
  executeFromEoa,
}: {
  status: "pending" | "approved" | "rejected";
  setStatus: (status: "pending" | "approved" | "rejected") => void;
  sendPipelineForExecution?: () => void;
  executeFromEoa?: () => void;
}) {
  const { t } = useTranslation();

  const Container = ({ children }: { children: React.ReactNode }) => {
    return <div className="flex gap-2">{children}</div>;
  };

  switch (status) {
    case "pending":
      return (
        <Container>
          <>
            <button
              onClick={sendPipelineForExecution || executeFromEoa}
              className="px-4 py-2 bg-green-500/20 hover:bg-green-500/30 text-green-300 rounded-lg transition-colors"
            >
              {t("pipelines.approve")}
            </button>
            <button
              onClick={() => setStatus("rejected")}
              className="px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-300 rounded-lg transition-colors"
            >
              {t("pipelines.reject")}
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
            <span>{t("pipelines.pipeline_scheduled_for_execution")}</span>
          </div>
        </Container>
      );
    case "rejected":
      return (
        <Container>
          <div className="text-red-400 flex items-center gap-2">
            <span>{t("pipelines.pipeline_rejected")}</span>
          </div>
        </Container>
      );
  }
}

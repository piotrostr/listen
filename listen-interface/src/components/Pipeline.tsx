import { usePrivy } from "@privy-io/react-auth";
import { useState } from "react";
import { config } from "../config";
import { useToast } from "../contexts/ToastContext";
import { Pipeline, PipelineActionType } from "../types/pipeline";
import { NotificationPipelineStep } from "./NotificationPipelineStep";
import { Spinner } from "./Spinner";
import { SwapPipelineStep } from "./SwapPipelineStep";

interface PipelineProps {
  pipeline: Pipeline;
}

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

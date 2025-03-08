import { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import {
  FaBan,
  FaCheckCircle,
  FaExternalLinkAlt,
  FaSpinner,
  FaTimesCircle,
} from "react-icons/fa";
import { PipelineCondition, PipelineConditionType } from "../types/pipeline";

interface PipelineStepContainerProps {
  children: React.ReactNode;
  conditions: PipelineCondition[];
  status?: "Pending" | "Completed" | "Failed" | "Cancelled";
  transactionHash: string | null;
  error: string | null;
  compact?: boolean;
}

export const PipelineStepContainer = ({
  children,
  conditions,
  status,
  transactionHash,
  error,
}: PipelineStepContainerProps) => {
  const { t } = useTranslation();
  return (
    <div className="border border-purple-500/30 rounded-lg lg:p-4 p-4 bg-black/40 backdrop-blur-sm">
      <div className="flex items-center gap-4">{children}</div>

      {/* Conditions */}
      {conditions.length > 0 && (
        <div className="mt-3 pt-3 border-t border-purple-500/30">
          <div className="text-sm text-purple-300">
            {t("pipelines.conditions")}
          </div>
          {conditions.map((condition, index) => (
            <div
              key={index}
              className="mt-1 lg:text-sm text-xs text-purple-200"
            >
              {condition.type === PipelineConditionType.Now
                ? t("pipelines.execute_immediately")
                : condition.type === PipelineConditionType.PriceAbove
                  ? `${t("pipelines.price_above")} ${condition.value} ${t("pipelines.for")} ${condition.asset.slice(0, 4)}...${condition.asset.slice(-4)}`
                  : `${t("pipelines.price_below")} ${condition.value} ${t("pipelines.for")} ${condition.asset.slice(0, 4)}...${condition.asset.slice(-4)}`}
            </div>
          ))}
        </div>
      )}
      {status && (
        <div className="mt-3 pt-3 border-t border-purple-500/30">
          <div className="text-sm text-purple-300">
            {t("pipelines.status")}:
          </div>
          <TransactionLink
            status={status}
            transactionHash={transactionHash}
            error={error}
          />
        </div>
      )}
    </div>
  );
};

const renderStatus = (status: string, t: TFunction) => {
  switch (status) {
    case "Pending":
      return (
        <span className="text-yellow-300 flex items-center gap-1">
          <FaSpinner /> {t("pipelines.pending")}
        </span>
      );
    case "Completed":
      return (
        <span className="text-green-300 flex items-center gap-1">
          <FaCheckCircle /> {t("pipelines.completed")}
        </span>
      );
    case "Failed":
      return (
        <span className="text-red-300 flex items-center gap-1">
          <FaTimesCircle /> {t("pipelines.failed")}
        </span>
      );
    case "Cancelled":
      return (
        <span className="text-gray-300 flex items-center gap-1">
          <FaBan /> {t("pipelines.cancelled")}
        </span>
      );
  }
};

function formatError(error: string, t: TFunction) {
  if (error.includes("insufficient funds")) {
    return t("pipelines.insufficient_balance");
  }
  if (error.includes("0x1771")) {
    return t("pipelines.slippage_tolerance_exceeded");
  }
  try {
    // Look for JSON between curly braces
    const match = error.match(/{.*}/);
    if (match) {
      const parsedError = JSON.parse(match[0]);
      if (parsedError?.error) {
        return JSON.stringify(parsedError.error);
      }
    }
    return error;
  } catch {
    return error;
  }
}

export const TransactionLink = ({
  status,
  transactionHash,
  error,
}: {
  status: string;
  transactionHash: string | null;
  error: string | null;
}) => {
  const { t } = useTranslation();
  return (
    <div className="text-xs sm:text-sm text-gray-400 flex flex-wrap items-center gap-2 mt-2">
      {renderStatus(status, t)}{" "}
      {transactionHash && (
        <span className="flex items-center gap-1 inline-flex">
          <a
            href={
              transactionHash.startsWith("0x")
                ? `https://blockscan.com/tx/${transactionHash}`
                : `https://solscan.io/tx/${transactionHash}`
            }
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-400 hover:text-blue-300 inline-flex items-center gap-1"
          >
            {transactionHash.slice(0, 6)}...{transactionHash.slice(-4)}
            <FaExternalLinkAlt size={10} />
          </a>
        </span>
      )}
      {error && (
        <span className="text-red-300 break-all overflow-hidden">
          {formatError(error, t)}
        </span>
      )}
    </div>
  );
};

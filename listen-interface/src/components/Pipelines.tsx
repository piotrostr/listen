import { usePrivy } from "@privy-io/react-auth";
import { useTranslation } from "react-i18next";
import { IoRefreshOutline } from "react-icons/io5";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";
import { usePipelines } from "../hooks/usePipelines";
import { ExtendedPipeline } from "../types/api";
import { ExtendedPipelineDisplay } from "./ExtendedPipelineDisplay";
import { Spinner } from "./Spinner";

interface PipelinesProps {
  statusFilter: string;
}

export function Pipelines({ statusFilter }: PipelinesProps) {
  const { ready } = usePrivy();
  const { isAuthenticated } = useIsAuthenticated();
  const { data, isLoading, error } = usePipelines();
  const { t } = useTranslation();

  if (!ready) {
    return (
      <div className="flex items-center justify-center h-full">
        <Spinner />
      </div>
    );
  }

  if (!isAuthenticated) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-400">
          {t("pipelines.please_connect_wallet")}
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Spinner />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-red-400">
          {error instanceof Error ? error.message : "Error loading pipelines"}
        </div>
      </div>
    );
  }

  const filteredPipelines = data?.pipelines.filter(
    (pipeline: ExtendedPipeline) => {
      if (statusFilter === "All") return true;
      return pipeline.status === statusFilter;
    }
  );

  return (
    <div className="h-full overflow-auto p-4">
      <div className="space-y-6">
        {filteredPipelines?.map((pipeline: ExtendedPipeline, index: number) => (
          <div
            key={`pipeline-${index}`}
            className="bg-black/40 backdrop-blur-sm border border-purple-500/30 rounded-lg p-4"
          >
            <ExtendedPipelineDisplay pipeline={pipeline} />
          </div>
        ))}
        {(!filteredPipelines || filteredPipelines.length === 0) && (
          <div className="text-center text-gray-400 py-8">
            {t("pipelines.no_pipelines_found")}
          </div>
        )}
      </div>
    </div>
  );
}

interface PipelinesHeaderProps {
  statusFilter: string;
  setStatusFilter: (filter: string) => void;
  onRefresh: () => void;
  isRefreshing: boolean;
}

export function PipelinesHeader({
  statusFilter,
  setStatusFilter,
  onRefresh,
  isRefreshing,
}: PipelinesHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 h-full">
      <select
        value={statusFilter}
        onChange={(e) => setStatusFilter(e.target.value)}
        className="bg-black/40 text-white border border-purple-500/30 rounded-lg px-4 h-8 text-sm"
      >
        <option value="All">{t("pipelines.all")}</option>
        <option value="Pending">{t("pipelines.pending")}</option>
        <option value="Completed">{t("pipelines.completed")}</option>
        <option value="Failed">{t("pipelines.failed")}</option>
      </select>
      <button
        onClick={onRefresh}
        disabled={isRefreshing}
        className="bg-black/40 text-white border border-purple-500/30 rounded-lg w-8 h-8 flex items-center justify-center hover:bg-purple-500/20 transition-colors disabled:opacity-50"
      >
        <IoRefreshOutline
          className={`w-4 h-4 ${isRefreshing ? "animate-spin" : ""}`}
        />
      </button>
    </div>
  );
}

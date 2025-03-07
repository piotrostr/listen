import { usePrivy } from "@privy-io/react-auth";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { IoRefreshOutline } from "react-icons/io5";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";
import { usePipelines } from "../hooks/usePipelines";
import { ExtendedPipeline } from "../types/api";
import { ExtendedPipelineDisplay } from "./ExtendedPipelineDisplay";
import { Spinner } from "./Spinner";

export function Pipelines() {
  const { ready } = usePrivy();
  const { isAuthenticated } = useIsAuthenticated();
  const { data, isLoading, error, refetch } = usePipelines();
  const [statusFilter, setStatusFilter] = useState<string>("All");
  const [isRefreshing, setIsRefreshing] = useState(false);

  const { t } = useTranslation();

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await refetch();
    setIsRefreshing(false);
  };

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
    <div className="container mx-auto lg:px-4 py-8">
      <div className="flex justify-between items-center mb-6 lg:p-0 p-4">
        <h1 className="lg:text-2xl text-xl font-bold text-white lg:text-left text-center">
          {t("pipelines.pipelines")}
        </h1>
        <div className="flex items-center gap-2">
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="bg-black/40 text-white border border-purple-500/30 rounded-lg px-4 py-2"
          >
            <option value="All">{t("pipelines.all")}</option>
            <option value="Pending">{t("pipelines.pending")}</option>
            <option value="Completed">{t("pipelines.completed")}</option>
            <option value="Failed">{t("pipelines.failed")}</option>
          </select>
          <button
            onClick={handleRefresh}
            disabled={isRefreshing}
            className="bg-black/40 text-white border border-purple-500/30 rounded-lg p-2 hover:bg-purple-500/20 transition-colors disabled:opacity-50"
          >
            <IoRefreshOutline
              className={`w-6 h-6 ${isRefreshing ? "animate-spin" : ""}`}
            />
          </button>
        </div>
      </div>
      <div className="space-y-6">
        {filteredPipelines?.map((pipeline: ExtendedPipeline, index: number) => (
          <div
            key={`pipeline-${index}`}
            className="bg-black/40 backdrop-blur-sm border border-purple-500/30 rounded-lg lg:p-6 p-2 py-4"
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

import { usePrivy } from "@privy-io/react-auth";
import { useQueryClient } from "@tanstack/react-query";
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

  const filteredPipelines = data?.pipelines.filter(
    (pipeline: ExtendedPipeline) => {
      if (statusFilter === "All") return true;
      return pipeline.status === statusFilter;
    }
  );

  return (
    <div className="h-full overflow-auto p-4 scrollable-container">
      {!ready || isLoading ? (
        <div className="flex items-center justify-center h-full">
          <Spinner />
        </div>
      ) : !isAuthenticated ? (
        <div className="flex items-center justify-center h-full">
          <div className="text-gray-400">
            {t("pipelines.please_connect_wallet")}
          </div>
        </div>
      ) : error ? (
        <div className="flex items-center justify-center h-full">
          <div className="text-red-400">
            {error instanceof Error ? error.message : "Error loading pipelines"}
          </div>
        </div>
      ) : (
        <div className="space-y-6">
          {filteredPipelines?.map(
            (pipeline: ExtendedPipeline, index: number) => (
              <div key={`pipeline-${index}`}>
                <ExtendedPipelineDisplay pipeline={pipeline} />
              </div>
            )
          )}
          {(!filteredPipelines || filteredPipelines.length === 0) && (
            <div className="text-center text-gray-400 py-8">
              {t("pipelines.no_pipelines_found")}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface PipelinesHeaderProps {
  statusFilter: string;
  setStatusFilter: (filter: string) => void;
}

export function PipelinesHeader({
  statusFilter,
  setStatusFilter,
}: PipelinesHeaderProps) {
  const { t } = useTranslation();

  const queryClient = useQueryClient();

  const refetch = async () => {
    await queryClient.resetQueries({
      queryKey: ["pipelines"],
    });
  };

  return (
    <div className="flex items-center gap-2 h-full">
      <select
        value={statusFilter}
        onChange={(e) => setStatusFilter(e.target.value)}
        className="bg-black/40 text-white border border-[#2D2D2D] rounded-lg px-4 h-8 text-sm"
      >
        <option value="All">{t("pipelines.all")}</option>
        <option value="Pending">{t("pipelines.pending")}</option>
        <option value="Completed">{t("pipelines.completed")}</option>
        <option value="Failed">{t("pipelines.failed")}</option>
      </select>
      <button
        onClick={refetch}
        className="bg-black/40 text-white border border-[#2D2D2D] rounded-lg w-8 h-8 flex items-center justify-center hover:bg-[#2D2D2D]"
      >
        <IoRefreshOutline className="w-4 h-4" />
      </button>
    </div>
  );
}

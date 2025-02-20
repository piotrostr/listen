import { usePrivy } from "@privy-io/react-auth";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";
import { usePipelines } from "../hooks/usePipelines";
import { ExtendedPipeline } from "../types/api";
import { ExtendedPipelineDisplay } from "./ExtendedPipelineDisplay";
import { Spinner } from "./Spinner";

export function Pipelines() {
  const { ready } = usePrivy();
  const { isAuthenticated } = useIsAuthenticated();
  const { data, isLoading, error } = usePipelines();

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
          Please connect your wallet to continue
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

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-white mb-6">Pipelines</h1>
      <div className="space-y-6">
        {data?.pipelines.map((pipeline: ExtendedPipeline, index: number) => (
          <div
            key={`pipeline-${index}`}
            className="bg-black/40 backdrop-blur-sm border border-purple-500/30 rounded-lg p-6"
          >
            <ExtendedPipelineDisplay pipeline={pipeline} />
          </div>
        ))}
        {(!data || data.pipelines.length === 0) && (
          <div className="text-center text-gray-400 py-8">
            No pipelines found
          </div>
        )}
      </div>
    </div>
  );
}

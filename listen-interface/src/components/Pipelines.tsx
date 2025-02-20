import { usePrivy } from "@privy-io/react-auth";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";

export function Pipelines() {
  const { isAuthenticated } = useIsAuthenticated();
  const { user } = usePrivy();

  if (!isAuthenticated || !user) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-gray-400">
          Please connect your wallet to continue
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-white mb-6">Pipelines</h1>
      <div className="bg-black/40 backdrop-blur-sm border border-purple-500/30 rounded-lg p-6">
        <p className="text-gray-300">
          Your automated trading pipelines will appear here.
        </p>
      </div>
    </div>
  );
}

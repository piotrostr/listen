import { FaExclamationTriangle } from "react-icons/fa";
import { z } from "zod";

export const HolderDistributionSchema = z.object({
  holder_distribution: z.array(
    z.object({
      address: z.string(),
      balance: z.number(),
    })
  ),
});

const MaxHolderSchema = z.object({
  address: z.string(),
  percentage: z.number(),
});

const IsolatedWalletsSchema = z.object({
  count: z.number(),
  total_percentage: z.number(),
});

const ClusterSchema = z.object({
  wallets: z.number(),
  percentage: z.number(),
});

const LinkedWalletsSchema = z.object({
  clusters: z.number(),
  total_percentage: z.number(),
  largest_clusters: z.array(ClusterSchema),
});

const DistributionSchema = z.object({
  gini_index: z.number(),
  top70_centralization: z.number(),
});

const HolderRiskSchema = z.object({
  isolated_wallets: IsolatedWalletsSchema,
  linked_wallets: LinkedWalletsSchema,
  distribution: DistributionSchema,
  risk_level: z.string(),
});

export const TokenHolderAnalysisSchema = z.object({
  status: z.string(),
  token_address: z.string().nullable(),
  updated_at: z.string().nullable(),
  max_holder: MaxHolderSchema.nullable(),
  holder_risk: HolderRiskSchema,
});

export type HolderDistribution = z.infer<typeof HolderDistributionSchema>;
export type TokenHolderAnalysis = z.infer<typeof TokenHolderAnalysisSchema>;

export const BubbleMapDisplay = ({
  topHolderAnalysis,
}: {
  topHolderAnalysis: TokenHolderAnalysis;
}) => {
  if (!topHolderAnalysis.token_address) {
    return (
      <div className="text-gray-400">
        <FaExclamationTriangle /> No token address provided
      </div>
    );
  }

  const formatPercentage = (num: number) => `${num.toFixed(2)}%`;

  return (
    <div className="rounded-lg px-2 py-1 lg:px-4 lg:py-3 my-2 backdrop-blur-sm">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Risk Overview Section */}
        <div className="p-4 rounded-lg border border-[#2D2D2D]">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-medium">Risk Overview</h3>
            <span
              className={`px-3 py-1 rounded-full text-sm ${
                topHolderAnalysis.holder_risk.risk_level === "High"
                  ? "bg-red-500/20 text-red-400"
                  : topHolderAnalysis.holder_risk.risk_level === "Moderate"
                    ? "bg-yellow-500/20 text-yellow-400"
                    : "bg-green-500/20 text-green-400"
              }`}
            >
              {topHolderAnalysis.holder_risk.risk_level} Risk
            </span>
          </div>

          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-400">Gini Index</span>
              <span>
                {formatPercentage(
                  topHolderAnalysis.holder_risk.distribution.gini_index
                )}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Top 70% Centralization</span>
              <span>
                {formatPercentage(
                  topHolderAnalysis.holder_risk.distribution
                    .top70_centralization
                )}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Max Holder</span>
              <span>
                {formatPercentage(
                  topHolderAnalysis.max_holder?.percentage || 0
                )}
              </span>
            </div>
          </div>
        </div>

        {/* Wallet Distribution Section */}
        <div className="p-4 rounded-lg border border-[#2D2D2D]">
          <h3 className="text-lg font-medium mb-4">Wallet Distribution</h3>
          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-400">Isolated Wallets</span>
              <span>
                {topHolderAnalysis.holder_risk.isolated_wallets.count} (
                {formatPercentage(
                  topHolderAnalysis.holder_risk.isolated_wallets
                    .total_percentage
                )}
                )
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Linked Clusters</span>
              <span>
                {topHolderAnalysis.holder_risk.linked_wallets.clusters} (
                {formatPercentage(
                  topHolderAnalysis.holder_risk.linked_wallets.total_percentage
                )}
                )
              </span>
            </div>
          </div>

          {/* Largest Clusters */}
          <h4 className="text-sm font-medium mt-4 mb-2">Largest Clusters</h4>
          <div className="space-y-2">
            {topHolderAnalysis.holder_risk.linked_wallets.largest_clusters.map(
              (cluster, index) => (
                <div key={index} className="flex justify-between text-sm">
                  <span className="text-gray-400">
                    {cluster.wallets} wallets
                  </span>
                  <span>{formatPercentage(cluster.percentage)}</span>
                </div>
              )
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

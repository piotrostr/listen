import { usePrivy } from "@privy-io/react-auth";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { config } from "../config";
import { usePortfolioStore } from "../store/portfolioStore";
import { ExtendedPipelineResponse, ExtendedPipelineSchema } from "../types/api";
import { waitForTransaction } from "../utils/transactionMonitor";
import { useIsAuthenticated } from "./useIsAuthenticated";

const MAX_POLLING_ATTEMPTS = 20;
const POLLING_INTERVAL = 1000; // 1 second

// Track steps that we've seen in pending state
const PENDING_STEP_IDS = new Set<string>();
// Track transactions we're already monitoring
const MONITORING_TX_HASHES = new Set<string>();

export const usePipelines = () => {
  const { isAuthenticated } = useIsAuthenticated();
  const { getAccessToken, ready } = usePrivy();
  const refreshPortfolio = usePortfolioStore((state) => state.refreshPortfolio);

  // Use a local ref for debounce timing
  const pendingRefreshTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );

  // Debounced portfolio refresh to prevent multiple rapid refreshes
  const debouncedRefreshPortfolio = () => {
    if (pendingRefreshTimeoutRef.current) {
      clearTimeout(pendingRefreshTimeoutRef.current);
    }

    pendingRefreshTimeoutRef.current = setTimeout(() => {
      console.log("usePipelines: Debounced portfolio refresh triggered");
      refreshPortfolio();
      pendingRefreshTimeoutRef.current = null;
    }, 100);
  };

  const queryResult = useQuery<ExtendedPipelineResponse, Error>({
    queryKey: ["pipelines"],
    queryFn: async (): Promise<ExtendedPipelineResponse> => {
      console.debug("usePipelines queryFn: Starting fetch...");
      if (!ready || !isAuthenticated) {
        console.debug(
          "usePipelines queryFn: Aborting - Not ready or not authenticated."
        );
        throw new Error("Not authenticated");
      }

      const accessToken = await getAccessToken();
      if (!accessToken) {
        console.debug("usePipelines queryFn: Aborting - No access token.");
        throw new Error("No access token available");
      }

      console.debug("usePipelines queryFn: Fetching from API...");
      const url = `${config.engineEndpoint}/pipelines`;
      const response = await fetch(url, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${accessToken}`,
          "Content-Type": "application/json",
          Accept: "application/json",
        },
      });

      if (!response.ok) {
        console.error(
          "usePipelines queryFn: API fetch failed.",
          response.status,
          response.statusText
        );
        throw new Error(`Failed to fetch pipelines: ${response.statusText}`);
      }

      const responseData = await response.json();
      const pipelinesRaw = responseData.pipelines;

      if (!pipelinesRaw) {
        console.debug("usePipelines queryFn: No pipelines array in response.");
        return { pipelines: [], status: responseData.status || "ok" };
      }

      const pipelines = [];
      for (const pipeline of pipelinesRaw) {
        try {
          pipelines.push(ExtendedPipelineSchema.parse(pipeline));
        } catch (err) {
          console.error("Failed to parse pipeline:", pipeline, err);
        }
      }

      pipelines.sort((a, b) => {
        return (
          new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
        );
      });

      console.debug("usePipelines queryFn: Fetch successful.");
      return {
        pipelines: pipelines,
        status: responseData.status,
      };
    },
    enabled: ready && isAuthenticated,
    refetchInterval: (query) => {
      const data = query.state.data;
      if (!data) {
        console.debug(
          "usePipelines refetchInterval: No data yet, stopping poll."
        );
        return false;
      }
      const hasPendingSteps = data.pipelines.some((pipeline) =>
        Object.values(pipeline.steps).some((step) => step.status === "Pending")
      );
      return hasPendingSteps ? POLLING_INTERVAL : false;
    },
    refetchOnWindowFocus: true,
    retry: MAX_POLLING_ATTEMPTS,
  });

  useEffect(() => {
    if (!queryResult.data) return;

    // First, track all steps in pending state so we can monitor when they complete
    queryResult.data.pipelines.forEach((pipeline) => {
      Object.values(pipeline.steps).forEach((step) => {
        // Track new pending steps
        if (step.status === "Pending") {
          PENDING_STEP_IDS.add(step.id);
        }

        // Check for steps that were previously pending and are now completed
        if (
          step.status === "Completed" &&
          step.transaction_hash &&
          PENDING_STEP_IDS.has(step.id) && // We previously saw this step as pending
          !MONITORING_TX_HASHES.has(step.transaction_hash) // Not already monitoring
        ) {
          console.log(
            `usePipelines: Step ${step.id} transitioned from Pending to Completed`
          );

          // Remove from pending set, it's now completed
          PENDING_STEP_IDS.delete(step.id);

          // Only monitor for Order actions
          if ("Order" in step.action) {
            console.log(
              `usePipelines: Monitoring newly completed Order transaction ${step.transaction_hash}`
            );

            // Mark as monitoring
            MONITORING_TX_HASHES.add(step.transaction_hash);

            if (step.action.Order.from_chain_caip2.startsWith("solana")) {
              waitForTransaction(
                step.transaction_hash!,
                undefined,
                () => {
                  console.log(
                    `usePipelines: Transaction ${step.transaction_hash} confirmed, refreshing portfolio`
                  );
                  MONITORING_TX_HASHES.delete(step.transaction_hash!);
                  debouncedRefreshPortfolio();
                },
                (error) => {
                  console.error(`usePipelines: Transaction failed: ${error}`);
                  MONITORING_TX_HASHES.delete(step.transaction_hash!);
                }
              );
            } else if (step.action.Order.from_chain_caip2.startsWith("eip")) {
              // For EVM transactions, use timeout
              setTimeout(() => {
                console.log(`usePipelines: EVM transaction timeout complete`);
                MONITORING_TX_HASHES.delete(step.transaction_hash!);
                debouncedRefreshPortfolio();
              }, 2000);
            }
          } else {
            console.log(
              `usePipelines: Completed step is not an Order, ignoring`
            );
          }
        }
      });
    });
  }, [queryResult.data, refreshPortfolio]);

  // Cleanup
  useEffect(() => {
    return () => {
      if (pendingRefreshTimeoutRef.current) {
        clearTimeout(pendingRefreshTimeoutRef.current);
      }
    };
  }, []);

  return queryResult;
};

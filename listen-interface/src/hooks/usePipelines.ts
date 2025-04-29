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

export const usePipelines = () => {
  const { isAuthenticated } = useIsAuthenticated();
  const { getAccessToken, ready } = usePrivy();
  const refreshPortfolio = usePortfolioStore((state) => state.refreshPortfolio);
  const processedCompletionTxHashesRef = useRef<Set<string>>(new Set());

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

    const currentData = queryResult.data;
    let transactionsToMonitor = false;

    currentData.pipelines.forEach((pipeline) => {
      Object.values(pipeline.steps).forEach((step) => {
        if (
          step.status === "Completed" &&
          step.transaction_hash &&
          !processedCompletionTxHashesRef.current.has(step.transaction_hash)
        ) {
          console.debug(
            `  useEffect: Detected completed step ${step.id} with unprocessed txHash ${step.transaction_hash}.`
          );
          // Mark this hash as processed immediately to prevent duplicate processing
          processedCompletionTxHashesRef.current.add(step.transaction_hash);
          console.debug(
            `    useEffect: Added ${step.transaction_hash} to processed ref. Current ref size:`,
            processedCompletionTxHashesRef.current.size
          );

          // Only monitor for Order actions
          if ("Order" in step.action) {
            console.debug(
              `    useEffect: Step ${step.id} is an Order. Starting transaction monitor.`
            );

            // Don't immediately refresh - instead wait for tx confirmation
            transactionsToMonitor = true;

            // Launch independent monitor for this transaction
            if (step.action.Order.from_chain_caip2.startsWith("solana")) {
              waitForTransaction(
                step.transaction_hash,
                undefined, // Use default RPC URL
                () => {
                  console.log(
                    `Transaction monitor: Transaction ${step.transaction_hash} confirmed, refreshing portfolio`
                  );
                  refreshPortfolio();
                },
                (error) => {
                  console.error(
                    `Transaction monitor: Failed to confirm transaction ${step.transaction_hash}: ${error}`
                  );
                }
              );
            } else if (step.action.Order.from_chain_caip2.startsWith("eip")) {
              // for now timeout 2 seconds and refetch the evm portfolio
              setTimeout(() => {
                console.log(
                  `Transaction monitor: ${step.transaction_hash}, refreshing portfolio but didn't wait for receipt`
                );
                refreshPortfolio();
              }, 2000);
            }
          } else {
            console.debug(
              `    useEffect: Step ${step.id} is not an Order. Skipping transaction monitoring.`
            );
          }
        }
      });
    });

    if (!transactionsToMonitor) {
      console.debug(
        "usePipelines useEffect: No transactions to monitor in this data update."
      );
    }
  }, [queryResult.data, refreshPortfolio]);

  return queryResult;
};

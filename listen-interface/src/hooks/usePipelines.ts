import { usePrivy } from "@privy-io/react-auth";
import { useEffect, useState } from "react";
import { config } from "../config";
import { useToast } from "../contexts/ToastContext";
import { ExtendedPipelineResponse, ExtendedPipelineSchema } from "../types/api";
import { useIsAuthenticated } from "./useIsAuthenticated";

export const usePipelines = () => {
  const { isAuthenticated } = useIsAuthenticated();
  const { getAccessToken, ready } = usePrivy();
  const { showToast } = useToast();
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<ExtendedPipelineResponse | null>(null);
  const [error, setError] = useState<Error | null>(null);

  const fetchPipelines = async () => {
    if (!ready || !isAuthenticated) return;

    setIsLoading(true);
    setError(null);

    try {
      const accessToken = await getAccessToken();

      if (!accessToken) {
        throw new Error("No access token available");
      }

      const url = `${config.API_BASE_URL}/v1/engine/pipelines`;
      const response = await fetch(url, {
        method: "GET",
        headers: {
          Authorization: `Bearer ${accessToken}`,
          "Content-Type": "application/json",
          Accept: "application/json",
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch pipelines: ${response.statusText}`);
      }

      const responseData = await response.json();
      const pipelinesRaw = responseData.pipelines;

      if (!pipelinesRaw) {
        throw new Error("No pipelines found");
      }

      const pipelines = [];
      for (const pipeline of pipelinesRaw) {
        try {
          pipelines.push(ExtendedPipelineSchema.parse(pipeline));
        } catch (err) {
          console.error(err);
        }
      }

      pipelines.sort((a, b) => {
        return (
          new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
        );
      });

      setData({
        pipelines: pipelines,
        status: responseData.status,
      });
    } catch (err) {
      const error =
        err instanceof Error ? err : new Error("Failed to fetch pipelines");
      setError(error);
      showToast(error.message, "error");
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchPipelines();

    // Set up polling interval
    const intervalId = setInterval(fetchPipelines, 30000);

    return () => clearInterval(intervalId);
  }, [ready, isAuthenticated]);

  return {
    data,
    isLoading,
    error,
    refetch: fetchPipelines,
  };
};

import { usePrivy } from "@privy-io/react-auth";
import { useQuery } from "@tanstack/react-query";
import { config } from "../config";
import { ExtendedPipelineResponse, ExtendedPipelineSchema } from "../types/api";
import { useIsAuthenticated } from "./useIsAuthenticated";

export const usePipelines = () => {
  const { isAuthenticated } = useIsAuthenticated();
  const { getAccessToken, ready } = usePrivy();

  return useQuery({
    queryKey: ["pipelines"],
    queryFn: async (): Promise<ExtendedPipelineResponse> => {
      if (!ready || !isAuthenticated) {
        throw new Error("Not authenticated");
      }

      const accessToken = await getAccessToken();

      if (!accessToken) {
        throw new Error("No access token available");
      }

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

      return {
        pipelines: pipelines,
        status: responseData.status,
      };
    },
    enabled: ready && isAuthenticated,
    staleTime: 30000, // Consider data fresh for 30 seconds
    refetchOnWindowFocus: true,
  });
};

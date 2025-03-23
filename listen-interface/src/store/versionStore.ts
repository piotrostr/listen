import { z } from "zod";
import { create } from "zustand";

const VersionResponseSchema = z.object({
  version: z.string(),
});

const POLL_INTERVAL = 15000;

export const CURRENT_VERSION = "2.5.0";

interface VersionState {
  version: string;
  isLoading: boolean;
  error: string | null;
  latestVersion: string;
  setVersion: (version: string) => void;
  startPolling: () => void;
  stopPolling: () => void;
}

export const useVersionStore = create<VersionState>((set) => {
  let pollingInterval: number | null = null;

  const fetchLatestVersion = async () => {
    try {
      set({ isLoading: true, error: null });
      const response = await fetch(
        "https://api.listen-rs.com/v1/adapter/version"
      );

      if (!response.ok) {
        throw new Error(`API request failed with status ${response.status}`);
      }

      const data = await response.json();

      // Validate the response data
      const validatedData = VersionResponseSchema.parse(data);

      set({
        latestVersion: validatedData.version,
        isLoading: false,
        error: null,
      });
    } catch (error) {
      console.error("Failed to fetch version:", error);
      set({
        isLoading: false,
        error: error instanceof Error ? error.message : "Unknown error",
      });
    }
  };

  return {
    version: CURRENT_VERSION,
    isLoading: false,
    error: null,
    latestVersion: "",
    setVersion: (version) => set({ version }),

    startPolling: () => {
      // Fetch immediately on start
      fetchLatestVersion();

      // Clear any existing interval to avoid duplicates
      if (pollingInterval !== null) {
        window.clearInterval(pollingInterval);
      }

      // Start polling every 15 seconds
      pollingInterval = window.setInterval(fetchLatestVersion, POLL_INTERVAL);
    },

    stopPolling: () => {
      if (pollingInterval !== null) {
        window.clearInterval(pollingInterval);
        pollingInterval = null;
      }
    },
  };
});

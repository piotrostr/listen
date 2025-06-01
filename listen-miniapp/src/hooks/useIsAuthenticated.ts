import { usePrivy } from "@privy-io/react-auth";
import { worldchainEnabled } from "../config/env";
import {
  userHasDelegatedEvmWallet,
  userHasDelegatedSolanaWallet,
} from "../lib/util";
import { useWorldAuth } from "./useWorldLogin";

export const useIsAuthenticated = () => {
  const { authenticated, ready, user } = usePrivy();
  const isDelegatedSolana = userHasDelegatedSolanaWallet(user);
  const isDelegatedEvm = userHasDelegatedEvmWallet(user);
  const { worldUserAddress } = useWorldAuth();

  if (worldchainEnabled) {
    // In development mode or testing mode, only check if we have a worldUserAddress
    const isDevMode = process.env.NODE_ENV === "development";
    const isTestingOverride = window.location.search.includes(
      "test-worldcoin=true"
    );
    const isWorldAuthenticated =
      isDevMode || isTestingOverride
        ? typeof worldUserAddress === "string"
        : typeof worldUserAddress === "string" && user !== null;

    return {
      isAuthenticated: isWorldAuthenticated,
      hasSolanaWallet: isDelegatedSolana,
      hasEvmWallet: isDelegatedEvm,
      ready,
      isLoading: !ready,
    };
  }
  return {
    isAuthenticated: ready && authenticated,
    hasSolanaWallet: isDelegatedSolana,
    hasEvmWallet: isDelegatedEvm,
    ready,
    isLoading: !ready,
  };
};

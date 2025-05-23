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

  console.log("worldUserAddress", worldUserAddress);

  if (worldchainEnabled) {
    // In development mode, only check if we have a worldUserAddress
    const isDevMode = process.env.NODE_ENV === "development";
    const isWorldAuthenticated = isDevMode
      ? typeof worldUserAddress === "string"
      : typeof worldUserAddress === "string" && user !== null;

    return {
      isAuthenticated: isWorldAuthenticated,
      hasSolanaWallet: false,
      hasEvmWallet: true,
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

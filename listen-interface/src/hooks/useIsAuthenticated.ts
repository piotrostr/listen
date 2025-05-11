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
    return {
      isAuthenticated: typeof worldUserAddress === "string",
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

import { usePrivy } from "@privy-io/react-auth";
import { useWorld } from "../contexts/WorldContext";
import {
  userHasDelegatedEvmWallet,
  userHasDelegatedSolanaWallet,
} from "../lib/util";
import { useWorldAuth } from "./useWorldLogin";

export const useIsAuthenticated = () => {
  const { authenticated, ready, user } = usePrivy();
  const isDelegatedSolana = userHasDelegatedSolanaWallet(user);
  const isDelegatedEvm = userHasDelegatedEvmWallet(user);
  const { isWorldApp } = useWorld();
  const { worldUserAddress } = useWorldAuth();

  if (isWorldApp) {
    return {
      isAuthenticated: worldUserAddress !== null,
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

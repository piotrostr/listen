import { usePrivy } from "@privy-io/react-auth";
import {
  userHasDelegatedEvmWallet,
  userHasDelegatedSolanaWallet,
} from "./util";

export const useIsAuthenticated = () => {
  const { authenticated, ready, user } = usePrivy();
  const isDelegatedSolana = userHasDelegatedSolanaWallet(user);
  const isDelegatedEvm = userHasDelegatedEvmWallet(user);

  return {
    isAuthenticated: ready && authenticated,
    hasSolanaWallet: isDelegatedSolana,
    hasEvmWallet: isDelegatedEvm,
    ready,
    isLoading: !ready,
  };
};

import { useLoginWithSiwe } from "@privy-io/react-auth";
import { useMutation, useQuery } from "@tanstack/react-query";
import { MiniKit } from "@worldcoin/minikit-js";
import { type Address } from "viem";
import { DEVELOPMENT_FALLBACK_ADDRESS } from "../config/env";

export const useWorldAuth = () => {
  // Check for development mode OR testing override
  const isDevMode = process.env.NODE_ENV === "development";
  const isTestingOverride = window.location.search.includes(
    "test-worldcoin=true"
  );

  // In development or testing mode, return the fallback address
  if (isDevMode || isTestingOverride) {
    return {
      worldLogin: () => {},
      isLoading: false,
      error: null,
      worldUserAddress: DEVELOPMENT_FALLBACK_ADDRESS,
      isUserLoading: false,
      userError: null,
    };
  }

  const { generateSiweNonce, loginWithSiwe } = useLoginWithSiwe();

  const mutation = useMutation({
    mutationFn: async () => {
      // Get nonce from Privy
      const privyNonce = await generateSiweNonce();

      // Use nonce with Worldcoin walletAuth
      const { finalPayload } = await MiniKit.commandsAsync.walletAuth({
        nonce: privyNonce,
      });

      if (finalPayload.status === "error") {
        throw new Error("Login failed");
      }

      // Complete SIWE flow with Privy
      const { message, signature } = finalPayload;
      const user = await loginWithSiwe({ message, signature });

      // Store the address for future use
      if (user?.wallet?.address) {
        localStorage.setItem("worldUserLoginAddress", user.wallet.address);
      }

      return user;
    },
  });

  const userQuery = useQuery({
    queryKey: ["user"],
    queryFn: () => {
      const storedAddress = localStorage.getItem("worldUserLoginAddress");
      const walletAddress = MiniKit.user?.walletAddress;
      return (walletAddress || storedAddress || null) as Address | null;
    },
    refetchInterval: 1000,
    staleTime: 0,
    gcTime: Infinity,
    placeholderData: (previousData) => previousData,
  });

  const nullState = {
    worldLogin: () => {},
    isLoading: false,
    error: null,
    worldUserAddress: null,
    isUserLoading: false,
    userError: null,
  };

  try {
    const isInstalled = MiniKit.isInstalled();

    if (!isInstalled) {
      return nullState;
    }
  } catch (error) {
    return nullState;
  }

  return {
    worldLogin: mutation.mutate,
    isLoading: mutation.isPending,
    error: mutation.error,
    worldUserAddress: userQuery.data,
    isUserLoading: userQuery.isLoading,
    userError: userQuery.error,
  };
};

import { useLoginWithSiwe } from "@privy-io/react-auth";
import { useMutation, useQuery } from "@tanstack/react-query";
import { MiniKit } from "@worldcoin/minikit-js";
import { type Address } from "viem";
import { DEVELOPMENT_FALLBACK_ADDRESS } from "../config/env";

export const useWorldAuth = () => {
  // In development, return the fallback address
  if (process.env.NODE_ENV === "development") {
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
      // Get the user's address from MiniKit
      const address = MiniKit.user?.walletAddress;
      if (!address) {
        throw new Error("No wallet address available");
      }

      // Get nonce from Privy
      const privyNonce = await generateSiweNonce({ address });

      // Use nonce with Worldcoin walletAuth
      const { finalPayload } = await MiniKit.commandsAsync.walletAuth({
        nonce: privyNonce,
        requestId: "0",
        expirationTime: new Date(
          new Date().getTime() + 7 * 24 * 60 * 60 * 1000
        ),
        notBefore: new Date(new Date().getTime() - 24 * 60 * 60 * 1000),
        statement: "Sign in with Ethereum to authenticate with our app",
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
    if (!MiniKit.isInstalled()) {
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

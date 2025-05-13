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

  const mutation = useMutation({
    mutationFn: async () => {
      const nonce = crypto.randomUUID().replace(/-/g, "");

      const { finalPayload } = await MiniKit.commandsAsync.walletAuth({
        nonce,
        requestId: "0",
        expirationTime: new Date(
          new Date().getTime() + 7 * 24 * 60 * 60 * 1000
        ),
        notBefore: new Date(new Date().getTime() - 24 * 60 * 60 * 1000),
        statement: "Sign in",
      });

      if (finalPayload.status === "error") {
        throw new Error("Login failed");
      }

      localStorage.setItem("worldUserLoginAddress", finalPayload.address);

      return finalPayload;
    },
  });

  const userQuery = useQuery({
    queryKey: ["user"],
    queryFn: () => {
      const storedAddress = localStorage.getItem("userWalletAddress");
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

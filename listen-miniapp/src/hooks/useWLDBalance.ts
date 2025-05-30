import { useQuery } from "@tanstack/react-query";
import { Address, erc20Abi, PublicClient } from "viem";
import { usePublicClient } from "wagmi";
import { useWorldAuth } from "./useWorldLogin";

const WLD_TOKEN_ADDRESS = "0x2cFc85d8E48F8EAB294be644d9E25C3030863003";

export async function getWLDBalance(
  publicClient: PublicClient,
  address: Address
) {
  if (!publicClient) return null;

  return publicClient.readContract({
    address: WLD_TOKEN_ADDRESS,
    abi: erc20Abi,
    functionName: "balanceOf",
    args: [address],
  });
}
export function useWLDBalance() {
  const publicClient = usePublicClient();
  const { worldUserAddress } = useWorldAuth();

  return useQuery({
    queryKey: ["wld-balance", worldUserAddress],
    queryFn: async () => {
      if (!worldUserAddress || !publicClient) return null;
      return getWLDBalance(publicClient, worldUserAddress as Address);
    },
    enabled: !!worldUserAddress && !!publicClient,
  });
}

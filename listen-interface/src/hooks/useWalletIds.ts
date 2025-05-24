import { LinkedAccountWithMetadata, useUser } from "@privy-io/react-auth";
import { useEffect, useState } from "react";

interface WalletWithId {
  id: string;
  address: string;
}

export const useWalletIds = () => {
  const { user } = useUser();
  const [walletIds, setWalletIds] = useState<Array<WalletWithId>>();

  console.log({ user });

  useEffect(() => {
    const walletIds = user?.linkedAccounts
      .map((wallet) => extractEmbeddedWallet(wallet))
      .filter((wallet) => wallet !== null);
    if (walletIds) {
      setWalletIds(walletIds);
    }
  }, [user]);

  return walletIds;
};

const extractEmbeddedWallet = (
  linkedAccount: LinkedAccountWithMetadata
): WalletWithId | null => {
  if (
    linkedAccount.type === "wallet" &&
    linkedAccount.walletClientType === "privy" &&
    linkedAccount.connectorType === "embedded" &&
    linkedAccount.id
  ) {
    return {
      id: linkedAccount.id,
      address: linkedAccount.address,
    };
  }
  return null;
};

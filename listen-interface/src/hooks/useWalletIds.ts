import { LinkedAccountWithMetadata, useUser } from "@privy-io/react-auth";
import { useEffect, useState } from "react";

export const useWalletIds = () => {
  const { user } = useUser();
  const [walletIds, setWalletIds] = useState<Array<string>>();

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

const extractEmbeddedWallet = (linkedAccount: LinkedAccountWithMetadata) => {
  if (
    linkedAccount.type === "wallet" &&
    linkedAccount.walletClientType === "privy" &&
    linkedAccount.connectorType === "embedded" &&
    linkedAccount.id
  ) {
    return linkedAccount.id;
  }
  return null;
};

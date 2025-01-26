import {
  usePrivy,
  useSolanaWallets,
  useDelegatedActions,
  type WalletWithMetadata,
} from "@privy-io/react-auth";

export function DelegateActionButton() {
  const { user } = usePrivy();
  const { ready, wallets, createWallet } = useSolanaWallets();
  const { delegateWallet } = useDelegatedActions();

  const onCreateWallet = async () => {
    const wallet = await createWallet();
    console.log(wallet);
  };

  // Find the embedded wallet to delegate from the array of the user's wallets
  const walletToDelegate = wallets.find(
    (wallet) => wallet.walletClientType === "privy",
  );

  // Check if the wallet to delegate by inspecting the user's linked accounts
  const isAlreadyDelegated = !!user?.linkedAccounts.find(
    (account): account is WalletWithMetadata =>
      account.type === "wallet" && account.delegated,
  );

  const delegatedWallet = user?.linkedAccounts.find(
    (account): account is WalletWithMetadata =>
      account.type === "wallet" && account.delegated,
  );

  const onDelegate = async () => {
    if (!walletToDelegate || !ready) return; // Button is disabled to prevent this case
    await delegateWallet({
      address: walletToDelegate.address,
      chainType: "solana",
    });
  };

  if (ready && !walletToDelegate) {
    return (
      <button disabled={!ready} onClick={onCreateWallet}>
        Create a wallet
      </button>
    );
  }

  return (
    <>
      {isAlreadyDelegated ? (
        <p>✅ {delegatedWallet?.address.slice(0, 5) + ".."}</p>
      ) : (
        <button
          disabled={!ready || !walletToDelegate || isAlreadyDelegated}
          onClick={onDelegate}
        >
          Delegate access
        </button>
      )}
    </>
  );
}

import { Connection, PublicKey } from "@solana/web3.js";
import { useEffect } from "react";
import { usePortfolioStore } from "../store/portfolioStore";
import { useWalletStore } from "../store/walletStore";
import { decodeTokenAccount } from "./util";

const connection = new Connection(import.meta.env.VITE_RPC_URL);

export function useTokenAccountsSubscription() {
  const { solanaAddress } = useWalletStore();
  const { updateTokenBalance } = usePortfolioStore();

  useEffect(() => {
    if (!solanaAddress) return;

    const owner = new PublicKey(solanaAddress);
    let subscriptionId: number | null = null;

    // TODO here probably need to subscribe to signature, signature needs to be pushed
    // from the engine upon success (privy webhooks are an option too but expensive)
    async function setupSubscription() {
      try {
        // Subscribe to all token accounts owned by the wallet
        subscriptionId = connection.onProgramAccountChange(
          new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), // Token program
          (accountInfo) => {
            try {
              console.log("UPDATE accountInfo", accountInfo);
              const decoded = decodeTokenAccount(accountInfo.accountInfo.data);
              if (!decoded) return;

              // Check if this account belongs to our wallet
              if (decoded.owner.toString() !== solanaAddress) return;

              const mint = decoded.mint.toString();
              const amount = Number(decoded.amount);

              // Update the balance in the store
              updateTokenBalance(mint, amount);
            } catch (error) {
              console.error("Error processing account update:", error);
            }
          },
          "confirmed",
          [
            {
              memcmp: {
                offset: 32, // Owner offset in token account data
                bytes: owner.toBase58(),
              },
            },
          ]
        );
      } catch (error) {
        console.error("Error setting up token account subscription:", error);
      }
    }

    setupSubscription();

    return () => {
      if (subscriptionId !== null) {
        connection.removeProgramAccountChangeListener(subscriptionId);
      }
    };
  }, [solanaAddress]);
}

import { Connection, TransactionSignature } from "@solana/web3.js";

/**
 * Wait for a Solana transaction to be confirmed using signatureSubscribe
 *
 * @param signature The transaction signature to monitor
 * @param rpcUrl The Solana RPC URL (default: from env)
 * @param onSuccess Callback to execute when transaction confirms successfully
 * @param onError Callback to execute if transaction fails
 * @returns A promise that resolves when the transaction completes
 */
export const waitForTransaction = async (
  signature: TransactionSignature,
  rpcUrl: string = import.meta.env.VITE_RPC_URL,
  onSuccess?: () => void,
  onError?: (error: string) => void
): Promise<boolean> => {
  if (!signature) {
    console.error("Transaction monitor: No signature provided");
    onError?.("No signature provided");
    return false;
  }

  console.debug(`Transaction monitor: Subscribing to signature ${signature}`);
  const connection = new Connection(rpcUrl);

  return new Promise((resolve) => {
    // Create timeout for safety
    const timeoutId = setTimeout(() => {
      console.warn("Transaction monitor: Subscription timed out");
      onError?.("Transaction confirmation timeout");
      resolve(false);
    }, 60000); // 1 minute timeout

    // Subscribe to transaction status
    const subscriptionId = connection.onSignature(
      signature,
      (result, context) => {
        clearTimeout(timeoutId);

        if (result.err) {
          console.error(
            `Transaction monitor: Transaction failed: ${JSON.stringify(result.err)}`
          );
          onError?.(
            typeof result.err === "string"
              ? result.err
              : JSON.stringify(result.err)
          );
          resolve(false);
          return;
        }

        console.debug(
          `Transaction monitor: Transaction confirmed! Slot: ${context.slot}`
        );
        onSuccess?.();
        resolve(true);
      },
      "processed"
    );

    // Also check if the transaction is already confirmed
    connection
      .getSignatureStatus(signature)
      .then((status) => {
        if (status && status.value) {
          // If already confirmed, clean up the subscription
          try {
            connection.removeSignatureListener(subscriptionId);
          } catch (err) {
            console.warn("Error removing signature listener:", err);
          }

          clearTimeout(timeoutId);

          if (status.value.err) {
            console.error(
              `Transaction monitor: Transaction already failed: ${JSON.stringify(status.value.err)}`
            );
            onError?.(
              typeof status.value.err === "string"
                ? status.value.err
                : JSON.stringify(status.value.err)
            );
            resolve(false);
            return;
          }

          if (
            status.value.confirmationStatus === "confirmed" ||
            status.value.confirmationStatus === "finalized"
          ) {
            console.debug(
              `Transaction monitor: Transaction already confirmed! Status: ${status.value.confirmationStatus}`
            );
            onSuccess?.();
            resolve(true);
          }
        }
      })
      .catch((err) => {
        console.warn("Error checking initial status:", err);
      });
  });
};

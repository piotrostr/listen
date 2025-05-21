import { useEffect } from "react";
import { config } from "../config";
import { useToast } from "../contexts/ToastContext";
import { useWalletIds } from "../hooks/useWalletIds";
import { getBalanceChange } from "../lib/balance-change";
import { useTokenStore } from "../store/tokenStore";
import { PriceUpdate } from "../types/price";

interface TransactionUpdate {
  [key: string]: string;
}

export const WebsocketInitializer = () => {
  const walletIds = useWalletIds();
  const { showToast } = useToast();
  const updateTokenData = useTokenStore.getState().updateTokenData;

  useEffect(() => {
    if (!walletIds) {
      return;
    }
    const solanaWallet = walletIds.find(
      (wallet) => !wallet.address.startsWith("0x")
    );
    const evmWallet = walletIds.find((wallet) =>
      wallet.address.startsWith("0x")
    );
    if (!solanaWallet) {
      return;
    }
    const ws = new WebSocket("wss://api.listen-rs.com/v1/adapter/ws");

    console.log(
      `Subscribing to ${config.adapterWsEndpoint} (wallet IDs: ${solanaWallet.id}, ${evmWallet?.id})`
    );

    ws.onmessage = async (event) => {
      try {
        const data = JSON.parse(event.data);
        if ("event" in data) {
          const transactionUpdate: TransactionUpdate = data;
          console.log("Transaction update", transactionUpdate);

          // Only process non-EVM transactions
          if (!transactionUpdate.signature.startsWith("0x")) {
            const balanceChange = await getBalanceChange(
              transactionUpdate.signature,
              solanaWallet.address
            );

            // Only show toast for positive balance changes
            if (balanceChange.uiAmount !== "0") {
              showToast(
                `+${balanceChange.uiAmount} ${balanceChange.symbol}`,
                "success"
              );
            }
          }
          return;
        }
        if ("mint" in data) {
          const priceUpdate: PriceUpdate = data;
          updateTokenData(priceUpdate);
          return;
        }
      } catch (error) {
        console.error("Error parsing message:", error);
      }
    };

    const wallet_ids = [solanaWallet.id];
    if (evmWallet) {
      wallet_ids.push(evmWallet.id);
    }

    ws.onopen = () => {
      ws.send(
        JSON.stringify({
          action: "subscribe",
          mints: ["*"],
          wallet_ids,
        })
      );
    };

    ws.onerror = (error) => {
      console.error("WebSocket failed:", error);
    };

    ws.onclose = () => {
      // Optionally implement reconnection logic here
    };

    return () => {
      ws.close();
    };
  }, [walletIds]);

  return null;
};

import { config } from "../config";
import { useTokenStore } from "../store/tokenStore";
import { PriceUpdate } from "../types/price";

interface TransactionUpdate {
  [key: string]: string;
}

export function setupWebSocket(walletIds: string[]) {
  const updateTokenData = useTokenStore.getState().updateTokenData;

  const ws = new WebSocket(config.adapterWsEndpoint);

  console.log(
    `Subscribing to ${config.adapterWsEndpoint} (wallet IDs: ${walletIds})`
  );

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if ("event" in data) {
        const transactionUpdate: TransactionUpdate = data;
        console.log("Transaction update", transactionUpdate);
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

  ws.onopen = () => {
    ws.send(
      JSON.stringify({
        action: "subscribe",
        mints: ["*"],
        wallet_ids: walletIds,
      })
    );
  };

  ws.onerror = (error) => {
    console.error("WebSocket failed:", error);
  };

  ws.onclose = () => {
    // Optionally implement reconnection logic here
  };

  return ws;
}

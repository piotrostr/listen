import { useTokenStore } from "../store/tokenStore";
import type { PriceUpdate } from "../types/price";

export function setupWebSocket() {
  const updateTokenData = useTokenStore.getState().updateTokenData;

  const ws = new WebSocket("wss://api.listen-rs.com/v1/adapter/ws");

  ws.onmessage = (event) => {
    try {
      const data: PriceUpdate = JSON.parse(event.data);
      updateTokenData(data);
    } catch (error) {
      console.error("Error parsing message:", error);
    }
  };

  ws.onopen = () => {
    ws.send(
      JSON.stringify({
        action: "subscribe",
        mints: ["*"],
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

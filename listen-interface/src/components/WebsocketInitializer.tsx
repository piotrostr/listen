import { useEffect } from "react";
import { useWalletIds } from "../hooks/useWalletIds";
import { setupWebSocket } from "../services/websocketService";

export const WebsocketInitializer = () => {
  const walletIds = useWalletIds();
  useEffect(() => {
    if (!walletIds) {
      return;
    }
    const ws = setupWebSocket(walletIds);
    return () => {
      ws.close();
    };
  }, [walletIds]);

  return null;
};

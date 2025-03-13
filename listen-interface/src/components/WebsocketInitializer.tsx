import { useEffect } from "react";
import { setupWebSocket } from "../services/websocketService";

export const WebsocketInitializer = () => {
  // Setup WebSocket connection
  useEffect(() => {
    const ws = setupWebSocket();
    return () => {
      ws.close();
    };
  }, []);

  return null;
};

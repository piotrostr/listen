export const config = {
  engineEndpoint:
    process.env.NODE_ENV === "production"
      ? "https://api.listen-rs.com/v1/engine"
      : "http://localhost:6966",
  kitEndpoint:
    process.env.NODE_ENV === "production"
      ? "https://api.listen-rs.com/v1/kit"
      : "http://localhost:6969",
  adapterEndpoint:
    process.env.NODE_ENV === "production"
      ? "https://api.listen-rs.com/v1/adapter"
      : "http://localhost:6967",
  adapterWsEndpoint:
    process.env.NODE_ENV === "production"
      ? "wss://api.listen-rs.com/v1/adapter/ws"
      : "ws://localhost:6967/ws",
};

const USE_LISTEN_DOMAIN_ENDPOINTS = false;

const ENGINE_PROD_ENDPOINT = USE_LISTEN_DOMAIN_ENDPOINTS
  ? "https://api.listen-rs.com/v1/engine"
  : "https://listen-engine.fly.dev";

const KIT_PROD_ENDPOINT = USE_LISTEN_DOMAIN_ENDPOINTS
  ? "https://api.listen-rs.com/v1/kit"
  : "https://listen-kit.fly.dev";

export const config = {
  engineEndpoint:
    process.env.NODE_ENV === "production"
      ? ENGINE_PROD_ENDPOINT
      : "http://localhost:6966",
  kitEndpoint:
    process.env.NODE_ENV === "production"
      ? KIT_PROD_ENDPOINT
      : "http://localhost:6969",
  adapterEndpoint: "https://api.listen-rs.com/v1/adapter",
  // process.env.NODE_ENV === "production"
  //   ? "https://api.listen-rs.com/v1/adapter"
  //   : "http://localhost:6968",
  adapterWsEndpoint:
    process.env.NODE_ENV === "production"
      ? "wss://api.listen-rs.com/v1/adapter/ws"
      : "ws://localhost:6968/ws",
};

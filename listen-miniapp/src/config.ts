export const config = {
  engineEndpoint:
    process.env.NODE_ENV === "production"
      ? "https://api.listen-rs.com/v1/engine"
      : "http://localhost:6966",
  kitEndpoint:
    process.env.NODE_ENV === "production"
      ? "https://api.listen-rs.com/v1/kit"
      : "http://localhost:6969",
};

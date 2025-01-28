export const config = {
  API_BASE_URL:
    process.env?.NODE_ENV === "production"
      ? "https://api.listen-rs.com"
      : "http://localhost:8080",
};

export const config = {
  API_BASE_URL:
    import.meta?.env?.NODE_ENV === "production"
      ? "https://api.listen-rs.com"
      : "http://localhost:6969",
};

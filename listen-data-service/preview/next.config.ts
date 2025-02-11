import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  experimental: {
    serverActions: {
      allowedOrigins: ["*"],
    },
  },
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "ipfs.io",
        pathname: "/ipfs/**",
      },
      {
        protocol: "https",
        hostname: "gateway.pinata.cloud",
        pathname: "/ipfs/**",
      },
      {
        protocol: "https",
        hostname: "gateway.irys.xyz",
        pathname: "/**",
      },
      {
        protocol: "https",
        hostname: "web3-static.socrates.com",
        pathname: "/**",
      },
      {
        protocol: "https",
        hostname: "nullinfo.app",
        pathname: "/**",
      },
    ],
  },
};

export default nextConfig;

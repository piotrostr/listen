import { TokenMetadataSchema } from "@/app/types";
import { Redis } from "ioredis";
import { NextApiRequest, NextApiResponse } from "next";

export default async function handler(
  request: NextApiRequest,
  res: NextApiResponse
) {
  const { mint } = request.query;

  if (!process.env.REDIS_URL) {
    return res.status(500).json({ error: "Redis URL not configured" });
  }

  const redis = new Redis(process.env.REDIS_URL);
  const key = `solana:${mint}`;

  try {
    const data = await redis.get(key);
    if (!data) {
      return res.status(404).json({ error: "Metadata not found" });
    }

    const parsedData = JSON.parse(data);
    const metadata = TokenMetadataSchema.parse(parsedData);
    return res.status(200).json(metadata);
  } catch (parseError) {
    console.error("Failed to parse metadata:", parseError);
    return res.status(500).json({ error: "Invalid metadata format" });
  } finally {
    redis.disconnect();
  }
}

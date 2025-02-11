import { TokenMetadataSchema } from "@/app/types";
import { Redis } from "ioredis";

export async function GET(
  _request: Request,
  { params }: { params: Promise<{ mint: string }> }
) {
  const { mint } = await params;

  if (!process.env.REDIS_URL) {
    return Response.json(
      { error: "Redis URL not configured" },
      { status: 500 }
    );
  }

  const redis = new Redis(process.env.REDIS_URL);
  const key = `solana:metadata:${mint}`;

  try {
    const data = await redis.get(key);
    if (!data) {
      return Response.json({ error: "Metadata not found" }, { status: 404 });
    }

    const parsedData = JSON.parse(data);
    const metadata = TokenMetadataSchema.parse(parsedData);
    return Response.json(metadata, { status: 200 });
  } catch (parseError) {
    console.error("Failed to parse metadata:", parseError);
    return Response.json({ error: "Invalid metadata format" }, { status: 500 });
  } finally {
    redis.disconnect();
  }
}

import { PriceUpdateSchema } from "@/app/types";
import { Redis } from "ioredis";

export async function GET() {
  if (!process.env.REDIS_URL) {
    return Response.json(
      { error: "Redis URL not configured" },
      { status: 500 }
    );
  }

  const redis = new Redis(process.env.REDIS_URL);
  const encoder = new TextEncoder();

  const stream = new ReadableStream({
    async start(controller) {
      try {
        await redis.subscribe("price_updates");

        redis.on("message", async (_channel, message) => {
          try {
            const data = PriceUpdateSchema.parse(JSON.parse(message));
            const encodedData = encoder.encode(
              `data: ${JSON.stringify(data)}\n\n`
            );
            controller.enqueue(encodedData);
          } catch (err) {
            console.error("Failed to parse message:", err);
          }
        });

        redis.on("error", (err) => {
          console.error("Redis error:", err);
          controller.error(err);
        });
      } catch (err) {
        console.error("Failed to subscribe:", err);
        controller.error(err);
        redis.disconnect();
      }
    },
    cancel() {
      redis.disconnect();
    },
  });

  return new Response(stream, {
    headers: {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
      // Add CORS headers
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET",
      "Access-Control-Allow-Headers": "Content-Type",
    },
  });
}

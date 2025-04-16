import { Elysia } from "elysia";
import { z } from "zod";
import { logger } from "./logger";
import { makeMemory } from "./memory";
import { AddMemorySchema, SearchFiltersSchema } from "./types";

// Define request body types using Zod
const SearchRequestSchema = z.object({
  query: z.string(),
  filters: SearchFiltersSchema,
});

type SearchRequest = z.infer<typeof SearchRequestSchema>;
type AddMemoryRequest = z.infer<typeof AddMemorySchema>;

const app = new Elysia();
const memory = await makeMemory();

// Health check
app.get("/health", () => {
  logger.info({ path: "/health" }, "Health check requested");
  return { status: "ok" };
});

// Add memory
app.post("/memories", async ({ body }: { body: AddMemoryRequest }) => {
  logger.debug({ path: "/memories", request: body }, "Raw request received");
  logger.info({ path: "/memories", operation: "add" }, "Adding new memory");

  const parsed = AddMemorySchema.safeParse(body);
  if (!parsed.success) {
    logger.warn(
      { path: "/memories", error: parsed.error },
      "Invalid memory data"
    );
    return new Response(JSON.stringify({ error: parsed.error }), {
      status: 400,
    });
  }

  try {
    const result = await memory.add(parsed.data.messages, {
      userId: parsed.data.config.user_id,
    });
    logger.debug({ path: "/memories", response: result }, "Raw response");
    logger.info(
      { path: "/memories", operation: "add", success: true },
      "Memory added successfully"
    );
    return result;
  } catch (error) {
    logger.error({ path: "/memories", error }, "Error adding memory");
    return new Response(JSON.stringify({ error: error }), {
      status: 500,
    });
  }
});

// Search memories
app.post("/memories/search", async ({ body }: { body: SearchRequest }) => {
  logger.debug(
    { path: "/memories/search", request: body },
    "Raw request received"
  );
  logger.info(
    { path: "/memories/search", query: body.query },
    "Searching memories"
  );

  const parsed = SearchRequestSchema.safeParse(body);
  if (!parsed.success) {
    logger.warn(
      { path: "/memories/search", error: parsed.error },
      "Invalid search request"
    );
    return new Response(JSON.stringify({ error: parsed.error }), {
      status: 400,
    });
  }

  const result = await memory.search(parsed.data.query, {
    filters: {
      userId: parsed.data.filters.user_id,
    },
  });
  logger.debug({ path: "/memories/search", response: result }, "Raw response");
  logger.info(
    { path: "/memories/search", results: result.results?.length || 0 },
    "Search completed"
  );

  return { result: result.results?.slice(0, 5) };
});

// Get memory by ID
app.get("/memories/:id", async ({ params }: { params: { id: string } }) => {
  logger.debug(
    { path: "/memories/:id", request: params },
    "Raw request received"
  );
  logger.info(
    { path: "/memories/:id", id: params.id },
    "Fetching memory by ID"
  );

  const result = await memory.get(params.id);
  if (!result) {
    logger.warn({ path: "/memories/:id", id: params.id }, "Memory not found");
    return new Response(JSON.stringify({ error: "Memory not found" }), {
      status: 404,
    });
  }
  logger.debug({ path: "/memories/:id", response: result }, "Raw response");
  logger.info(
    { path: "/memories/:id", id: params.id, success: true },
    "Memory retrieved"
  );
  return result;
});

async function main() {
  app.listen(9696);
  logger.info({ port: 9696 }, "🚀 Server started");
}

main();

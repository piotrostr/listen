import { Elysia } from "elysia";
import { z } from "zod";
import { assertEnv, makeMemory } from "./memory";
import { AddMemorySchema, SearchFiltersSchema } from "./types";

// Define request body types using Zod
const SearchRequestSchema = z.object({
  query: z.string(),
  filters: SearchFiltersSchema.optional(),
});

type SearchRequest = z.infer<typeof SearchRequestSchema>;
type AddMemoryRequest = z.infer<typeof AddMemorySchema>;

const app = new Elysia();
const memory = await makeMemory();

// Health check
app.get("/health", () => ({ status: "ok" }));

// Add memory
app.post("/memories", async ({ body }: { body: AddMemoryRequest }) => {
  const parsed = AddMemorySchema.safeParse(body);
  if (!parsed.success) {
    return new Response(JSON.stringify({ error: parsed.error }), {
      status: 400,
    });
  }

  const result = await memory.add(parsed.data.messages, parsed.data.config);
  return result;
});

// Search memories
app.post("/memories/search", async ({ body }: { body: SearchRequest }) => {
  const parsed = SearchRequestSchema.safeParse(body);
  if (!parsed.success) {
    return new Response(JSON.stringify({ error: parsed.error }), {
      status: 400,
    });
  }

  const result = await memory.search(parsed.data.query, {
    filters: parsed.data.filters,
  });
  return result;
});

// Get memory by ID
app.get("/memories/:id", async ({ params }: { params: { id: string } }) => {
  const result = await memory.get(params.id);
  if (!result) {
    return new Response(JSON.stringify({ error: "Memory not found" }), {
      status: 404,
    });
  }
  return result;
});

async function main() {
  assertEnv;
  app.listen(9696);
  console.log("🚀 Server running on http://localhost:9696");
}

main();

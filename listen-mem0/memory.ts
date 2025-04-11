import { QdrantClient } from "@qdrant/js-client-rest";
import { Memory } from "mem0ai/oss";

export const assertEnv = () => {
  if (!process.env.GEMINI_API_KEY) {
    throw new Error("GEMINI_API_KEY is not set");
  }

  if (!process.env.QDRANT_URL) {
    throw new Error("QDRANT_URL is not set");
  }

  if (!process.env.QDRANT_COLLECTION_NAME) {
    throw new Error("QDRANT_COLLECTION_NAME is not set");
  }
};

export const makeMemory = async () => {
  const collectionName = "memory-mem0";
  const host = "127.0.0.1";
  const port = 6333;

  assertEnv();

  // ensure collection is created
  const qdrant = new QdrantClient({
    host,
    port,
  });
  if (!(await qdrant.collectionExists(collectionName))) {
    let ok = await qdrant.createCollection(collectionName, {
      vectors: {
        size: 768,
        distance: "Cosine",
      },
    });
    if (!ok) {
      throw new Error(`Failed to create collection ${collectionName}`);
    }
  }
  if (!(await qdrant.collectionExists("memory_migrations"))) {
    const ok = await qdrant.createCollection("memory_migrations", {
      vectors: {
        size: 768,
        distance: "Cosine",
      },
    });
    if (!ok) {
      throw new Error(`Failed to create collection memory_migrations`);
    }
  }

  const memory = new Memory({
    // customPrompt: "",
    // disableHistory: true, // TODO make it true and possibly centralize
    embedder: {
      provider: "google",
      config: {
        apiKey: process.env.GEMINI_API_KEY,
        model: "text-embedding-004",
        // url: "TODO",
      },
    },
    vectorStore: {
      provider: "qdrant",
      config: {
        collectionName,
        dimension: 768,
        host,
        port,
      },
    },
    disableHistory: true,
    llm: {
      provider: "google",
      config: {
        apiKey: process.env.GEMINI_API_KEY,
        model: "gemini-2.0-flash",
      },
    },
  });
  return memory;
};

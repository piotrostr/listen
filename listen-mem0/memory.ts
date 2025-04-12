import { QdrantClient } from "@qdrant/js-client-rest";
import { Memory } from "mem0ai/oss";
import { logger } from "./logger";

export const AGENT_ID = "681385B2-FC6A-49C4-9033-189A09EE306A";

export const ensureEnv = () => {
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

export const ensureCollections = async () => {
  console.log("URL:", process.env.QDRANT_URL);
  console.log("COLLECTION NAME:", process.env.QDRANT_COLLECTION_NAME);

  const qdrant = new QdrantClient({
    url: process.env.QDRANT_URL,
    ...(process.env.QDRANT_API_KEY
      ? { apiKey: process.env.QDRANT_API_KEY }
      : {}),
  });

  if (!(await qdrant.collectionExists(process.env.QDRANT_COLLECTION_NAME!))) {
    let ok = await qdrant.createCollection(
      process.env.QDRANT_COLLECTION_NAME!,
      {
        vectors: {
          size: 768,
          distance: "Cosine",
        },
      }
    );
    if (!ok) {
      throw new Error(
        `Failed to create collection ${process.env.QDRANT_COLLECTION_NAME}`
      );
    }
  } else {
    logger.info(
      `Collection ${process.env.QDRANT_COLLECTION_NAME} already exists`
    );
  }

  if (!(await qdrant.collectionExists("memory_migrations"))) {
    let ok = await qdrant.createCollection("memory_migrations", {
      vectors: {
        size: 768,
        distance: "Cosine",
      },
    });
    if (!ok) {
      throw new Error(`Failed to create collection memory_migrations`);
    }
  } else {
    logger.info(`Collection memory_migrations already exists`);
  }
};

export const makeMemory = async () => {
  ensureEnv();
  ensureCollections();

  const memory = new Memory({
    // customPrompt: "", // TODO
    embedder: {
      provider: "google",
      config: {
        apiKey: process.env.GEMINI_API_KEY,
        model: "text-embedding-004",
      },
    },
    vectorStore: {
      provider: "qdrant",
      config: {
        collectionName: process.env.QDRANT_COLLECTION_NAME!,
        dimension: 768,
        url: process.env.QDRANT_URL,
        ...(process.env.QDRANT_API_KEY
          ? { apiKey: process.env.QDRANT_API_KEY }
          : {}),
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
    // TODO enable this possibly
    // enableGraph: true,
    //graphStore: {
    //  provider: "neo4j",
    //  config: {
    //    url: "bolt://localhost:7687",
    //    username: "neo4j",
    //    password: "password",
    //  },
    //},
  });
  return memory;
};

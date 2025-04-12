import { QdrantClient } from "@qdrant/js-client-rest";
import { Memory } from "mem0ai/oss";
import neo4j from "neo4j-driver";
import { logger } from "./logger";
import { customPrompt } from "./prompts";

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

  if (!process.env.NEO4J_URL) {
    throw new Error("NEO4J_URL is not set");
  }

  if (!process.env.NEO4J_USERNAME) {
    throw new Error("NEO4J_USERNAME is not set");
  }

  if (!process.env.NEO4J_PASSWORD) {
    throw new Error("NEO4J_PASSWORD is not set");
  }
};

export const ensureNeo4j = async () => {
  const driver = neo4j.driver(
    process.env.NEO4J_URL!,
    neo4j.auth.basic(process.env.NEO4J_USERNAME!, process.env.NEO4J_PASSWORD!)
  );
  const serverInfo = await driver.getServerInfo();
  logger.info(`Neo4j server info: ${serverInfo}`);
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
  ensureNeo4j();

  // const model = "gemini-2.0-flash";
  const model = "gpt-4o-mini";
  const embeddingModel = "text-embedding-004";
  const embeddingDimensions = 768;

  const memory = new Memory({
    disableHistory: true,
    enableGraph: true,
    customPrompt,
    embedder: {
      provider: "google",
      config: {
        apiKey: process.env.GEMINI_API_KEY,
        model: embeddingModel,
      },
    },
    vectorStore: {
      provider: "qdrant",
      config: {
        collectionName: process.env.QDRANT_COLLECTION_NAME!,
        dimension: embeddingDimensions,
        url: process.env.QDRANT_URL,
        ...(process.env.QDRANT_API_KEY
          ? { apiKey: process.env.QDRANT_API_KEY }
          : {}),
      },
    },
    llm: {
      provider: "openai",
      config: {
        apiKey: process.env.OPENAI_API_KEY,
        model: model,
      },
    },
    graphStore: {
      provider: "neo4j",
      config: {
        url: process.env.NEO4J_URL!,
        username: process.env.NEO4J_USERNAME!,
        password: process.env.NEO4J_PASSWORD!,
      },
      llm: {
        provider: "openai",
        config: {
          apiKey: process.env.OPENAI_API_KEY,
          model: model,
        },
      },
    },
  });
  return memory;
};

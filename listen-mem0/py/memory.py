import dotenv
import os
from mem0 import Memory
from mem0.configs.base import MemoryConfig, EmbedderConfig, VectorStoreConfig, GraphStoreConfig, LlmConfig
from qdrant_client import QdrantClient
import mem0
from neo4j import GraphDatabase
from logger import logger
from prompts import custom_prompt

dotenv.load_dotenv()

AGENT_ID = "681385B2-FC6A-49C4-9033-189A09EE306A"

def ensure_env():
  if not os.getenv("GEMINI_API_KEY"):
    raise Exception("GEMINI_API_KEY is not set")

  if not os.getenv("QDRANT_URL"):
    raise Exception("QDRANT_URL is not set")

  if not os.getenv("QDRANT_COLLECTION_NAME"):
    raise Exception("QDRANT_COLLECTION_NAME is not set")

  if not os.getenv("NEO4J_URL"):
    raise Exception("NEO4J_URL is not set")

  if not os.getenv("NEO4J_USERNAME"):
    raise Exception("NEO4J_USERNAME is not set")

  if not os.getenv("NEO4J_PASSWORD"):
    raise Exception("NEO4J_PASSWORD is not set")

def ensure_neo4j():
  driver = GraphDatabase.driver(
    os.getenv("NEO4J_URL"),
    auth=(os.getenv("NEO4J_USERNAME"), os.getenv("NEO4J_PASSWORD"))
  );
  serverInfo = driver.get_server_info()
  logger.info(f"Neo4j server info: {serverInfo}")

def ensure_collections():
  logger.info(f"URL: {os.getenv('QDRANT_URL')}")
  logger.info(f"COLLECTION NAME: {os.getenv('QDRANT_COLLECTION_NAME')}")

  qdrant = QdrantClient(
    url=os.getenv("QDRANT_URL"),
    api_key=os.getenv("QDRANT_API_KEY")
  )

  if not qdrant.collection_exists(os.getenv("QDRANT_COLLECTION_NAME")):
    ok = qdrant.create_collection(
      os.getenv("QDRANT_COLLECTION_NAME"),
      vectors_config={
        "size": 768,
        "distance": "Cosine",
      },
    );
    if not ok:
      raise Exception(
        f"Failed to create collection {os.getenv('QDRANT_COLLECTION_NAME')}"
      );
  else:
    logger.info(
      f"Collection {os.getenv('QDRANT_COLLECTION_NAME')} already exists"
    );


  if not qdrant.collection_exists("memory_migrations"):
    ok = qdrant.create_collection(
      "memory_migrations",
      vectors_config={
        "size": 768,
        "distance": "Cosine",
      },
    );
    if not ok:
      raise Exception(
        f"Failed to create collection memory_migrations"
      );
  else:
    logger.info(
      f"Collection memory_migrations already exists"
    );

def makeMemoryManaged():
  return mem0.MemoryClient(api_key=os.getenv("MEM0_API_KEY"))


def make_memory():
  ensure_env();
  ensure_collections();
  ensure_neo4j();

  model = "models/gemini-2.0-flash";
  embeddingModel = "models/text-embedding-004";
  # embeddingDimensions = 768;

  memory = Memory.from_config({
    "vectorStore": {
      "provider": "qdrant",
			"config": {
				"url": os.getenv("QDRANT_URL"),
				"api_key": os.getenv("QDRANT_API_KEY"),
			}
    },
    "llm": {
      "provider": "openai",
      "config": {
        "api_key": os.getenv("OPENAI_API_KEY"),
        "model": "gpt-4o-mini",
      },
    },
    "embedder": {
      "provider": "openai",
			"config": {
				"model": "text-embedding-3-small",
				"api_key": os.getenv("OPENAI_API_KEY"),
			}
    },
    "graphStore": {
      "provider": "neo4j",
      "config": {
        "url": os.getenv("NEO4J_URL"),
        "username": os.getenv("NEO4J_USERNAME"),
        "password": os.getenv("NEO4J_PASSWORD"),
      },
    },
  })
  return memory


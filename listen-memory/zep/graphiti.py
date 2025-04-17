import os
from datetime import datetime
from graphiti_core import Graphiti
from graphiti_core.llm_client.gemini_client import GeminiClient, LLMConfig
from graphiti_core.embedder.gemini import GeminiEmbedder, GeminiEmbedderConfig
from graphiti_core.nodes import EpisodeType
from graphiti_core.utils.bulk_utils import RawEpisode
import logging
import sys
import json


def setup_logging():
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(logging.INFO)
    formatter = logging.Formatter("%(name)s - %(levelname)s - %(message)s")
    console_handler.setFormatter(formatter)
    logger.addHandler(console_handler)
    return logger


logger = setup_logging()


def make_graphiti():
    api_key = os.getenv("GEMINI_API_KEY")
    api_key = os.getenv("GEMINI_API_KEY")

    graphiti = Graphiti(
        "bolt://localhost:7687",
        "neo4j",
        "password",
        llm_client=GeminiClient(
            config=LLMConfig(api_key=api_key, model="gemini-2.0-flash")
        ),
        embedder=GeminiEmbedder(
            config=GeminiEmbedderConfig(
                api_key=api_key, embedding_model="embedding-001"
            )
        ),
    )
    return graphiti


def read_in_sample_tool_outputs():
    path = os.getenv("HOME") + "/solana/listen/listen-kit/tool_output_samples/"
    logger.info(f"Reading tool outputs from: {path}")
    files = os.listdir(path)
    logger.info(f"Found files: {files}")
    contents = []
    for file in files:
        file_path = os.path.join(path, file)
        try:
            with open(file_path, "r") as f:
                contents.append(json.load(f))
        except Exception as e:
            logger.error(f"Error reading file {file_path}: {e}")
    return contents


async def insert(graphiti: Graphiti):
    await graphiti.build_indices_and_constraints()
    tool_outputs = read_in_sample_tool_outputs()[:1]

    processed_count = 0
    error_count = 0

    for i, output_data in enumerate(tool_outputs):
        if not output_data:
            logger.warning(f"Skipping empty string/file: {output_data}")
            continue  # Skip empty strings/files

        print(output_data)
        tool_name = output_data.get("tool_name", "Unknown Tool")
        episode = RawEpisode(
            name=f"Tool Call: {tool_name}",
            content=str(output_data),  # Store parsed JSON object
            source=EpisodeType.json,
            source_description="Sample Tool Output Ingestion (Tool JSON)",
            reference_time=datetime.now(),
        )

        # Add episode individually
        logger.info(f"Attempting to add episode {i+1} (Name: {episode.name})...")
        await graphiti.add_episode(
            name=episode.name,
            episode_body=episode.content,  # Pass the parsed JSON object
            source=episode.source,
            source_description=episode.source_description,
            reference_time=episode.reference_time,
            # You might need to add other relevant parameters if your RawEpisode uses them
        )
        logger.info(f"Successfully added episode {i+1}.")
        processed_count += 1

    logger.info(f"Finished processing. Added: {processed_count}, Errors: {error_count}")


async def query(graphiti: Graphiti, prompt: str):
    response = await graphiti.search(prompt)
    print(response)

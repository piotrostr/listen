import hashlib
import numpy as np
from dataclasses import dataclass
from typing import List
import voyageai

from google import genai
from google.genai import types
import os
import sqlite3

client = genai.Client(api_key=os.environ['GEMINI_API_KEY'])

def setup_embedding_db():
    """Set up SQLite database for storing embeddings"""
    conn = sqlite3.connect('embeddings.db')
    cursor = conn.cursor()
    
    # Create table if it doesn't exist
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS embeddings (
        prompt_hash TEXT PRIMARY KEY,
        prompt TEXT,
        embedding BLOB,
        count INTEGER DEFAULT 1
    )
    ''')
    
    conn.commit()
    return conn

@dataclass
class Embedding:
    embedding: np.ndarray
    prompt: str

def embed(prompts: list[str]) -> List[Embedding]:
    """
    Embed a list of prompts using Voyage AI.
    
    Args:
        prompts: List of text strings to embed
        
    Returns:
        List of Embedding objects containing the embedding vectors and original prompts
    """
    # Create Voyage AI client using API key from environment variable
    voyage_client = voyageai.Client(api_key=os.environ.get('VOYAGE_API_KEY'))
    
    # Get embeddings from Voyage AI
    result = voyage_client.embed(
        prompts, 
        model="voyage-3",
        input_type="document"
    )
    
    # Convert to our Embedding format
    embeddings = []
    for i, embedding_vector in enumerate(result.embeddings):
        embeddings.append(Embedding(
            embedding=np.array(embedding_vector, dtype=np.float32),
            prompt=prompts[i]
        ))
    
    return embeddings

def embed_gemini(prompts: list[str]) -> List[Embedding]:
    """Embed a prompt using Gemini API"""
    result = client.models.embed_content(
        model="gemini-embedding-exp-03-07",
        contents=prompts,
    )
    embeddings = []
    for embedding, prompt in zip(result.embeddings, prompts):
        embeddings.append(Embedding(embedding=np.array(embedding.values, dtype=np.float32), prompt=prompt))
    return embeddings

def get_embedding(text, conn):
    """Get embedding from database or Gemini API"""
    cursor = conn.cursor()
    
    # Create a hash of the preprocessed text
    text_hash = hashlib.md5(text.encode()).hexdigest()
    
    # Check if embedding exists in database
    cursor.execute("SELECT embedding FROM embeddings WHERE prompt_hash = ?", (text_hash,))
    result = cursor.fetchone()
    
    if result:
        # Increment count for this prompt
        cursor.execute("UPDATE embeddings SET count = count + 1 WHERE prompt_hash = ?", (text_hash,))
        conn.commit()
        # Convert stored blob back to numpy array
        return np.frombuffer(result[0], dtype=np.float32)
    
    # If not in database, get from API
    embeddings = embed([text])  # This returns a list of Embedding objects
    
    if not embeddings or len(embeddings) == 0:
        raise ValueError(f"Failed to get embedding for text: {text}")
    
    # Get the first (and only) embedding from the list
    embedding = embeddings[0]
    
    # Store in database
    cursor.execute(
        "INSERT INTO embeddings (prompt_hash, prompt, embedding, count) VALUES (?, ?, ?, ?)",
        (text_hash, text, embedding.embedding.tobytes(), 1)
    )
    conn.commit()
    
    return embedding.embedding
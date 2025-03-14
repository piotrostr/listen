from dataclasses import dataclass
import pandas as pd
import json
import glob
import numpy as np
from pprint import pprint
import re
from sklearn.feature_extraction.text import TfidfVectorizer
from sentence_transformers import SentenceTransformer
from umap.umap_ import UMAP
import hdbscan
from sklearn.metrics import silhouette_score
import sqlite3
import hashlib

import pydantic
from typing import Optional, List, Dict, Any, Tuple
import os
from google import genai
from google.genai import types

client = genai.Client(api_key=os.environ['GEMINI_API_KEY'])

class Chat(pydantic.BaseModel):
    _id: Optional[str] = None
    user_id: Optional[str] = None
    wallet_address: Optional[str] = None
    pubkey: Optional[str] = None
    chat_request: Dict[str, Any]
    responses: List[Dict[str, Any]]

def load_chats(one_file: bool = False) -> List[Chat]:
    """Load all chat files from raw_chats directory and parse them into Chat objects."""
    all_chats = []
    
    for file_path in glob.glob("raw_chats/*.json"):
        try:
            with open(file_path, "r") as f:
                chats_chunk = json.load(f)
                
                # Handle both list and dict formats
                if isinstance(chats_chunk, dict):
                    chats_chunk = [chats_chunk]
                
                for chat_data in chats_chunk:
                    try:
                        chat = Chat(**chat_data)
                        all_chats.append(chat)
                    except Exception as e:
                        print(f"Error parsing chat: {e}")
        except Exception as e:
            print(f"Error loading file {file_path}: {e}")
        if one_file:
            break
    
    print(f"Loaded {len(all_chats)} chats successfully")
    return all_chats

def extract_chat_info(chat: Chat) -> Dict[str, Any]:
    """Extract relevant information from a Chat object."""
    # Basic info
    info = {
        'chat_id': chat._id,
        'user_id': chat.user_id,
        'wallet_address': chat.wallet_address,
        'pubkey': chat.pubkey,
    }
    
    # Extract data from chat_request
    if chat.chat_request:
        # Extract prompt
        info['prompt'] = chat.chat_request.get('prompt')
        
        # Extract chain information
        info['chain'] = chat.chat_request.get('chain')
        
        # Extract chat history
        chat_history = chat.chat_request.get('chat_history', [])
        info['chat_history_length'] = len(chat_history)
        
        # Extract user messages from chat history
        user_messages = [
            msg.get('content', []) 
            for msg in chat_history 
            if msg.get('role') == 'user'
        ]
        info['user_message_count'] = len(user_messages)
        
        # Extract preamble if available
        info['has_preamble'] = 'preamble' in chat.chat_request
    
    # Extract response content
    if chat.responses and len(chat.responses) > 0:
        info['response_content'] = chat.responses[0].get('content')
        info['response_type'] = chat.responses[0].get('type')
        info['response_count'] = len(chat.responses)
    
    return info

def create_chat_dataframe(chats: List[Chat]) -> pd.DataFrame:
    """Create a pandas DataFrame from a list of Chat objects."""
    chat_info_list = [extract_chat_info(chat) for chat in chats]
    return pd.DataFrame(chat_info_list)

def analyze_chats():
    """Main function to load, process and analyze chats."""
    chats = load_chats()
    
    if not chats:
        print("No chats loaded. Check the raw_chats directory.")
        return
    
    # Create DataFrame for analysis
    df = create_chat_dataframe(chats)
    
    # Print basic statistics
    print(f"\nTotal chats: {len(df)}")
    print(f"Unique users: {df['user_id'].nunique()}")
    
    if 'chain' in df.columns:
        print("\nChain distribution:")
        print(df['chain'].value_counts())
    
    # You can add more analysis here
    
    return df

def preprocess_text(text):
    """Enhanced crypto-aware preprocessing"""
    text = text.lower().strip()
    
    # Preserve crypto-specific patterns
    text = re.sub(r'\$[a-z]+', '[TOKEN]', text)  # Normalize token names
    text = re.sub(r'0x[a-f0-9]+', '[CONTRACT]', text)  # Contract addresses
    text = re.sub(r'\b[a-z0-9]{40,}\b', '[HASH]', text)  # Long hashes
    
    # Remove non-essential punctuation but keep question marks
    text = re.sub(r'[^\w\s$.-?]', '', text)
    return text

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
    
    # If not in database, get from Gemini API
    embedding = embed(text)
    
    # Store in database
    cursor.execute(
        "INSERT INTO embeddings (prompt_hash, prompt, embedding, count) VALUES (?, ?, ?, ?)",
        (text_hash, text, embedding.tobytes(), 1)
    )
    conn.commit()
    
    return embedding

def get_prompt_distribution(conn):
    """Get distribution of prompts based on count"""
    cursor = conn.cursor()
    cursor.execute("SELECT prompt, count FROM embeddings ORDER BY count DESC")
    results = cursor.fetchall()
    
    return pd.DataFrame(results, columns=['prompt', 'count'])

def cluster_prompts(prompts):
    """Enhanced clustering with semantic focus using Gemini embeddings"""
    # Clean and normalize prompts
    cleaned = [preprocess_text(p) for p in prompts]
    
    # Set up database connection
    conn = setup_embedding_db()
    
    # Get embeddings (from DB or API)
    vectors = []
    for text in cleaned:
        embedding = get_embedding(text, conn)
        vectors.append(embedding)
    
    vectors = np.array(vectors)
    
    # Print prompt distribution
    print("\nPrompt Distribution:")
    distribution = get_prompt_distribution(conn)
    print(distribution.head(10))  # Show top 10 most common prompts
    
    # Close connection
    conn.close()

    # Rest of the clustering pipeline remains the same...
    umap_embeddings = UMAP(
        n_neighbors=10,
        n_components=10,
        metric='cosine',
        min_dist=0.1,
        spread=1.0
    ).fit_transform(vectors)

    clusterer = hdbscan.HDBSCAN(
        min_cluster_size=3,
        min_samples=2,
        cluster_selection_epsilon=0.3,
        cluster_selection_method='leaf'
    )
    labels = clusterer.fit_predict(umap_embeddings)
    
    return labels

def analyze_embeddings():
    """Analyze the stored embeddings and their distribution"""
    conn = setup_embedding_db()
    
    # Get distribution
    distribution = get_prompt_distribution(conn)
    
    # Get total count of embeddings
    cursor = conn.cursor()
    cursor.execute("SELECT SUM(count) FROM embeddings")
    total = cursor.fetchone()[0] or 0
    
    print(f"\nTotal unique prompts: {len(distribution)}")
    print(f"Total prompt occurrences: {total}")
    
    # Show top prompts
    if not distribution.empty:
        print("\nTop 10 most common prompts:")
        for i, (prompt, count) in enumerate(distribution.head(10).values):
            print(f"{i+1}. '{prompt[:50]}...' - {count} occurrences ({count/total*100:.1f}%)")
    
    conn.close()
    return distribution

if __name__ == "__main__":
    df = analyze_chats()
    
    # If there are prompts in the dataframe, cluster them
    if 'prompt' in df.columns and not df['prompt'].isna().all():
        print("\nClustering prompts...")
        prompts = df['prompt'].dropna().tolist()
        labels = cluster_prompts(prompts)
        
        # Add cluster labels to dataframe
        cluster_df = pd.DataFrame({
            'prompt': df['prompt'].dropna(),
            'cluster': labels
        })
        print(f"\nFound {len(set(labels) - {-1})} clusters")
        
        # Analyze the embedding database
        analyze_embeddings()
    
    # Optional: save to CSV
    # df.to_csv("chat_analysis.csv", index=False)

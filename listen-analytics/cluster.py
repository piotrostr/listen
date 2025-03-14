from embed import setup_embedding_db, get_embedding
from preprocess import preprocess_text
import numpy as np
from umap.umap_ import UMAP
import hdbscan
import pandas as pd

def get_prompt_distribution(conn):
    """Get distribution of prompts based on count"""
    cursor = conn.cursor()
    cursor.execute("SELECT prompt, count FROM embeddings ORDER BY count DESC")
    results = cursor.fetchall()
    
    return pd.DataFrame(results, columns=['prompt', 'count'])

def cluster_prompts(prompts, min_cluster_size=3, min_samples=2, cluster_epsilon=0.3):
    """Enhanced clustering with semantic focus using pre-generated embeddings"""
    # Clean and normalize prompts
    cleaned = [preprocess_text(p) for p in prompts]
    
    # Set up database connection
    conn = setup_embedding_db()
    
    # Get all embeddings from the database at once
    cursor = conn.cursor()
    
    # Create a dictionary to map prompts to their embeddings
    prompt_to_embedding = {}
    
    # Fetch all embeddings from the database
    cursor.execute("SELECT prompt, embedding FROM embeddings")
    all_db_embeddings = cursor.fetchall()
    
    for prompt, embedding_blob in all_db_embeddings:
        prompt_to_embedding[preprocess_text(prompt)] = np.frombuffer(embedding_blob, dtype=np.float32)
    
    # Get embeddings for our prompts
    vectors = []
    for text in cleaned:
        if text in prompt_to_embedding:
            vectors.append(prompt_to_embedding[text])
        else:
            print(f"Warning: No embedding found for prompt: {text[:50]}...")
            # Use a zero vector as a fallback (or you could skip this prompt)
            vectors.append(np.zeros(1024, dtype=np.float32))
    
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
        min_cluster_size=min_cluster_size,
        min_samples=min_samples,
        cluster_selection_epsilon=cluster_epsilon,
        cluster_selection_method='leaf'
    )
    labels = clusterer.fit_predict(umap_embeddings)
    
    return labels

from embed import setup_embedding_db, get_embedding
from preprocess import preprocess_text
import numpy as np
from umap.umap_ import UMAP
import hdbscan

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

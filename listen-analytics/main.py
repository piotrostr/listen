from analyze import load_chats, cluster_prompts, extract_chat_info, analyze_embeddings
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
import numpy as np
import tqdm
import argparse
import sqlite3
import hashlib
import time
import random
from embed import embed, setup_embedding_db

def generate_embeddings(prompts, batch_size=128):
	"""Generate embeddings for prompts in batches with sequential processing"""
	# Ensure the database table exists
	conn = setup_embedding_db()
	cursor = conn.cursor()
	
	# Create progress bar
	progress_bar = tqdm.tqdm(total=len(prompts), desc="Generating embeddings")
	
	# Set to track unique prompts we've processed
	processed_hashes = set()
	processed_count = 0
	unique_count = 0
	
	# Process in batches to reduce database operations
	for i in range(0, len(prompts), batch_size):
		batch = prompts[i:i+batch_size]
		
		for prompt in batch:
			# Skip empty prompts
			if not prompt:
				continue
				
			# Create a hash of the prompt
			text_hash = hashlib.md5(prompt.encode()).hexdigest()
			
			# Skip if we've already processed this hash in the current run
			if text_hash in processed_hashes:
				progress_bar.update(1)
				processed_count += 1
				continue
				
			# Add to processed hashes
			processed_hashes.add(text_hash)
			unique_count += 1
			
			# Check if embedding exists in database
			cursor.execute("SELECT embedding FROM embeddings WHERE prompt_hash = ?", (text_hash,))
			result = cursor.fetchone()
			
			if not result:
				# If not in database, get from API
				try:
					embeddings = embed([prompt])
					if embeddings and len(embeddings) > 0:
						embedding = embeddings[0]
						# Store in database
						cursor.execute(
							"INSERT INTO embeddings (prompt_hash, prompt, embedding) VALUES (?, ?, ?)",
							(text_hash, prompt, embedding.embedding.tobytes())
						)
						conn.commit()
						# Simple rate limiting - just sleep a bit between API calls
						time.sleep(4.3)  # ~15rpm
				except Exception as e:
					print(f"Error embedding prompt: {e}")
					# Sleep a bit longer on error
					time.sleep(1)
			
			progress_bar.update(1)
			processed_count += 1
	
	progress_bar.close()
	conn.close()
	
	print(f"Embedding generation complete. Processed {processed_count} prompts with {unique_count} unique prompts.")
	return list(processed_hashes)

def sample_embeddings(n=5):
	"""Retrieve n random embedding entries from the database and display them"""
	conn = sqlite3.connect('embeddings.db')
	cursor = conn.cursor()
	
	# Get total count of embeddings
	cursor.execute("SELECT COUNT(*) FROM embeddings")
	total_count = cursor.fetchone()[0]
	
	if total_count == 0:
		print("No embeddings found in the database.")
		conn.close()
		return
	
	# Get n random entries
	cursor.execute("SELECT prompt_hash, prompt FROM embeddings ORDER BY RANDOM() LIMIT ?", (n,))
	samples = cursor.fetchall()
	
	# Create a DataFrame for display
	df = pd.DataFrame(samples, columns=['Hash', 'Prompt'])
	
	# Truncate long prompts for display
	df['Prompt'] = df['Prompt'].apply(lambda x: (x[:100] + '...') if len(x) > 100 else x)
	
	print(f"\nSample of {n} random embeddings from database (total: {total_count}):")
	print(df)
	
	conn.close()

def get_embedding_by_hash(hash_value):
	"""Retrieve and display a specific embedding by its hash value"""
	conn = sqlite3.connect('embeddings.db')
	cursor = conn.cursor()
	
	# Query for the specific hash
	cursor.execute("SELECT prompt_hash, prompt, embedding FROM embeddings WHERE prompt_hash = ?", (hash_value,))
	result = cursor.fetchone()
	
	if not result:
		print(f"No embedding found with hash: {hash_value}")
		conn.close()
		return
	
	# Unpack the result
	prompt_hash, prompt, embedding_bytes = result
	
	# Create a DataFrame for the basic info
	df = pd.DataFrame([{
		'Hash': prompt_hash,
		'Prompt': prompt
	}])
	
	print("\nEmbedding details:")
	print(df)
	
	# Convert embedding bytes to numpy array for visualization
	embedding_array = np.frombuffer(embedding_bytes, dtype=np.float32)
	
	# Show embedding statistics
	print(f"\nEmbedding vector (showing first 10 of {len(embedding_array)} dimensions):")
	print(embedding_array[:10])
	print(f"Min: {embedding_array.min():.6f}, Max: {embedding_array.max():.6f}, Mean: {embedding_array.mean():.6f}")
	
	conn.close()

if __name__ == "__main__":
	parser = argparse.ArgumentParser(description="Chat analysis and embedding generation")
	parser.add_argument("--generate-embeddings", action="store_true", help="Generate embeddings for all prompts")
	parser.add_argument("--analyze-embeddings", action="store_true", help="Analyze existing embeddings")
	parser.add_argument("--sample-embeddings", action="store_true", help="Display random sample of embeddings")
	parser.add_argument("--sample-size", type=int, default=5, help="Number of random embeddings to sample")
	parser.add_argument("--get-by-hash", type=str, help="Retrieve embedding by hash value")
	parser.add_argument("--batch-size", type=int, default=128, help="Batch size for embedding generation")
	args = parser.parse_args()
	
	if args.get_by_hash:
		# Get embedding by hash
		get_embedding_by_hash(args.get_by_hash)
	elif args.sample_embeddings:
		# Sample random embeddings from the database
		sample_embeddings(args.sample_size)
	elif args.generate_embeddings:
		print("Loading chats for embedding generation...")
		chats = load_chats()
		prompts = [chat.chat_request.get('prompt') for chat in chats if chat.chat_request.get('prompt')]
		print(f"Found {len(prompts)} prompts for embedding")
		
		# Run the embedding generation
		processed = generate_embeddings(prompts, args.batch_size)
		print(f"Embedding generation complete. Processed {len(processed)} prompts.")
		
		# Analyze embeddings after generation if requested
		if args.analyze_embeddings:
			print("\nAnalyzing embeddings...")
			analyze_embeddings()
	elif args.analyze_embeddings:
		# Just analyze embeddings without generating new ones
		print("Analyzing embeddings...")
		analyze_embeddings()
	else:
		# Original clustering code
		chats = load_chats(True)
		prompts = [chat.chat_request['prompt'] for chat in chats]
		
		# Perform clustering
		cluster_labels = cluster_prompts(prompts)
		
		# Create DataFrame for analysis
		df = pd.DataFrame({
			'prompt': prompts,
			'cluster': cluster_labels
		})
		
		# Display cluster groupings
		for cluster_id in sorted(df['cluster'].unique()):
			print(f"\nCluster {cluster_id}:")
			print(df[df['cluster'] == cluster_id]['prompt'].head().tolist())
		
		# Analyze cluster quality
		unique_clusters = df['cluster'].nunique()
		print(f"\nIdentified {unique_clusters} semantic clusters")
		
		# Show most representative prompts per cluster
		for cluster_id in sorted(df['cluster'].unique()):
			clustered_prompts = df[df['cluster'] == cluster_id]['prompt']
			print(f"\nCluster {cluster_id} ({len(clustered_prompts)} prompts):")
			
			# Find most characteristic prompt using TF-IDF
			vectorizer = TfidfVectorizer()
			tfidf = vectorizer.fit_transform(clustered_prompts)
			most_typical = clustered_prompts.iloc[np.argmax(tfidf.sum(axis=1))]
			print(f"Most representative: '{most_typical}'")
			
			print("Example prompts:")
			print(clustered_prompts.head(3).tolist())
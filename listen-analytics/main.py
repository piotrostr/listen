from analyze import load_chats, cluster_prompts, extract_chat_info, analyze_embeddings
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
import numpy as np
import tqdm
import argparse
import sqlite3
import time
import redis
from embed import embed, setup_embedding_db
import os

def generate_embeddings(prompts, batch_size=50, redis_port=6379):
	"""Generate embeddings for prompts in batches using a simple set-based approach"""
	# Ensure the database table exists
	conn = setup_embedding_db()
	cursor = conn.cursor()
	
	# Connect to Redis to use as a persistent set of processed prompts
	r = redis.Redis(host='localhost', port=redis_port, db=0)
	
	# Filter out empty prompts
	valid_prompts = [p for p in prompts if p]
	total_prompts = len(valid_prompts)
	
	# Create progress bar
	progress_bar = tqdm.tqdm(total=total_prompts, desc="Generating embeddings")
	
	# Track statistics
	processed_count = 0
	unique_count = 0
	
	# Process prompts in batches
	pending_prompts = []
	
	for prompt in valid_prompts:
		# Check if we've already processed this prompt (in Redis)
		if r.sismember('processed_prompts', prompt):
			progress_bar.update(1)
			processed_count += 1
			continue
		
		# Check if embedding exists in database
		cursor.execute("SELECT embedding FROM embeddings WHERE prompt = ?", (prompt,))
		result = cursor.fetchone()
		
		if result:
			# Already in database, mark as processed in Redis
			r.sadd('processed_prompts', prompt)
			progress_bar.update(1)
			processed_count += 1
			continue
		
		# Add to pending batch
		pending_prompts.append(prompt)
		
		# Process batch when it reaches the desired size
		if len(pending_prompts) >= batch_size:
			process_batch(pending_prompts, cursor, conn, r, progress_bar)
			processed_count += len(pending_prompts)
			unique_count += len(pending_prompts)
			pending_prompts = []
	
	# Process any remaining prompts
	if pending_prompts:
		process_batch(pending_prompts, cursor, conn, r, progress_bar)
		processed_count += len(pending_prompts)
		unique_count += len(pending_prompts)
	
	progress_bar.close()
	conn.close()
	
	# Get total count of processed prompts from Redis
	total_processed = r.scard('processed_prompts')
	
	print(f"Embedding generation complete. Processed {processed_count} prompts with {unique_count} new unique prompts.")
	print(f"Total unique prompts in Redis: {total_processed}")
	return unique_count

def process_batch(pending_prompts, cursor, conn, redis_client, progress_bar):
	"""Process a batch of pending prompts by sending them all at once"""
	try:
		# Send all prompts in a single API call
		embeddings = embed(pending_prompts)
		print(f"Generated {len(embeddings)} embeddings")
		
		if embeddings and len(embeddings) > 0:
			# Process each embedding and store in database
			for i, prompt in enumerate(pending_prompts):
				if i < len(embeddings):  # Safety check
					embedding = embeddings[i]
					# Store in database
					cursor.execute(
						"INSERT INTO embeddings (prompt, embedding) VALUES (?, ?)",
						(prompt, embedding.embedding.tobytes())
					)
					
					# Mark as processed in Redis
					redis_client.sadd('processed_prompts', prompt)
			
			# Commit all at once
			conn.commit()
			print(f"Committed {len(pending_prompts)} embeddings")
			
			# Simple rate limiting - just one sleep for the whole batch
			time.sleep(1)
	except Exception as e:
		print(f"Error embedding batch: {e}")
		# Sleep a bit longer on error
		time.sleep(1)
	
	# Update progress bar for the whole batch
	progress_bar.update(len(pending_prompts))

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
	cursor.execute("SELECT prompt FROM embeddings ORDER BY RANDOM() LIMIT ?", (n,))
	samples = cursor.fetchall()
	
	# Create a DataFrame for display
	df = pd.DataFrame(samples, columns=['Prompt'])
	
	# Truncate long prompts for display
	df['Prompt'] = df['Prompt'].apply(lambda x: (x[:100] + '...') if len(x) > 100 else x)
	
	print(f"\nSample of {n} random embeddings from database (total: {total_count}):")
	print(df)
	
	conn.close()

def get_embedding_by_prompt(prompt):
	"""Retrieve and display a specific embedding by its prompt"""
	conn = sqlite3.connect('embeddings.db')
	cursor = conn.cursor()
	
	# Query for the specific prompt
	cursor.execute("SELECT prompt, embedding FROM embeddings WHERE prompt = ?", (prompt,))
	result = cursor.fetchone()
	
	if not result:
		print(f"No embedding found for prompt: {prompt}")
		conn.close()
		return
	
	# Unpack the result
	prompt, embedding_bytes = result
	
	# Create a DataFrame for the basic info
	df = pd.DataFrame([{
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

def get_all_prompts_from_db():
	"""Get all prompts from the embeddings database"""
	conn = setup_embedding_db()
	cursor = conn.cursor()
	
	# Get all prompts from the database
	cursor.execute("SELECT prompt FROM embeddings")
	results = cursor.fetchall()
	
	# Extract prompts from results
	prompts = [row[0] for row in results]
	
	conn.close()
	return prompts

def export_clusters_to_files(df, output_dir="clusters"):
	"""Export clusters to text files, one prompt per line, with counts in filenames"""
	# Create output directory if it doesn't exist
	os.makedirs(output_dir, exist_ok=True)
	
	# Get unique clusters
	clusters = sorted(df['cluster'].unique())
	
	# Count clusters (excluding noise if present)
	valid_clusters = [c for c in clusters if c != -1]
	group_count = len(valid_clusters)
	
	# Export each cluster to a file
	for index, cluster_id in enumerate(valid_clusters):
		# Get prompts for this cluster
		cluster_prompts = df[df['cluster'] == cluster_id]['prompt'].tolist()
		prompt_count = len(cluster_prompts)
		
		# Include count in filename
		filename = f"{output_dir}/cluster-{prompt_count}-{index}-{group_count}.txt"
		
		# Write to file, one prompt per line
		with open(filename, 'w') as f:
			for prompt in cluster_prompts:
				f.write(f"{prompt}\n")
		
		print(f"Exported {prompt_count} prompts to {filename}")
	
	# Handle noise points separately if they exist
	if -1 in clusters:
		noise_prompts = df[df['cluster'] == -1]['prompt'].tolist()
		noise_count = len(noise_prompts)
		if noise_prompts:
			with open(f"{output_dir}/cluster-noise-{noise_count}.txt", 'w') as f:
				for prompt in noise_prompts:
					f.write(f"{prompt}\n")
			print(f"Exported {noise_count} noise prompts to {output_dir}/cluster-noise-{noise_count}.txt")
	
	return group_count

if __name__ == "__main__":
	parser = argparse.ArgumentParser(description="Chat analysis and embedding generation")
	parser.add_argument("--generate-embeddings", action="store_true", help="Generate embeddings for all prompts")
	parser.add_argument("--analyze-embeddings", action="store_true", help="Analyze existing embeddings")
	parser.add_argument("--sample-embeddings", action="store_true", help="Display random sample of embeddings")
	parser.add_argument("--sample-size", type=int, default=5, help="Number of random embeddings to sample")
	parser.add_argument("--get-by-prompt", type=str, help="Retrieve embedding by prompt text")
	parser.add_argument("--batch-size", type=int, default=100, help="Batch size for embedding generation")
	parser.add_argument("--redis-port", type=int, default=6379, help="Redis port")
	parser.add_argument("--use-db-only", action="store_true", help="Use only prompts from the database for clustering")
	parser.add_argument("--min-cluster-size", type=int, default=3, help="Minimum cluster size for HDBSCAN")
	parser.add_argument("--min-samples", type=int, default=2, help="Minimum samples parameter for HDBSCAN")
	parser.add_argument("--cluster-epsilon", type=float, default=0.3, help="Cluster selection epsilon for HDBSCAN")
	parser.add_argument("--export-clusters", action="store_true", help="Export clusters to text files")
	parser.add_argument("--output-dir", type=str, default="clusters", help="Directory for exported clusters")
	args = parser.parse_args()
	
	if args.get_by_prompt:
		# Get embedding by prompt
		get_embedding_by_prompt(args.get_by_prompt)
	elif args.sample_embeddings:
		# Sample random embeddings from the database
		sample_embeddings(args.sample_size)
	elif args.generate_embeddings:
		print("Loading chats for embedding generation...")
		chats = load_chats()
		prompts = [chat.chat_request.get('prompt') for chat in chats if chat.chat_request.get('prompt')]
		print(f"Found {len(prompts)} prompts for embedding")
		
		# Run the embedding generation
		processed = generate_embeddings(prompts, args.batch_size, args.redis_port)
		print(f"Embedding generation complete. Processed {processed} new unique prompts.")
		
		# Analyze embeddings after generation if requested
		if args.analyze_embeddings:
			print("\nAnalyzing embeddings...")
			analyze_embeddings()
	elif args.analyze_embeddings:
		# Just analyze embeddings without generating new ones
		print("Analyzing embeddings...")
		analyze_embeddings()
	else:
		# Get prompts for clustering
		if args.use_db_only:
			# Get prompts directly from the database
			print("Getting prompts from database for clustering...")
			prompts = get_all_prompts_from_db()
			print(f"Found {len(prompts)} prompts in database")
		else:
			# Original approach - load from chats
			chats = load_chats(True)
			prompts = [chat.chat_request['prompt'] for chat in chats]
		
		# Perform clustering
		cluster_labels = cluster_prompts(
			prompts,
			min_cluster_size=args.min_cluster_size,
			min_samples=args.min_samples,
			cluster_epsilon=args.cluster_epsilon
		)
		
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
		
		# After clustering and analysis
		if args.export_clusters:
			print("\nExporting clusters to text files...")
			export_clusters_to_files(df, args.output_dir)
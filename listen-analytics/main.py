from analyze import load_chats, cluster_prompts, extract_chat_info, analyze_embeddings
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
import numpy as np
import concurrent.futures
import threading
import tqdm
import argparse
import sqlite3
import hashlib
import time
import queue
import random
from embed import embed, setup_embedding_db

# Global token bucket rate limiter
class TokenBucketRateLimiter:
	def __init__(self, rate=3, capacity=3):
		self.rate = rate  # tokens per second
		self.capacity = capacity  # bucket size
		self.tokens = capacity  # start with a full bucket
		self.last_update = time.time()
		self.lock = threading.Lock()
	
	def _add_tokens(self):
		now = time.time()
		time_passed = now - self.last_update
		new_tokens = time_passed * self.rate
		
		if new_tokens > 0:
			self.tokens = min(self.capacity, self.tokens + new_tokens)
			self.last_update = now
	
	def acquire(self):
		with self.lock:
			self._add_tokens()
			
			if self.tokens >= 1:
				self.tokens -= 1
				return True
			else:
				# Calculate time to wait for next token
				wait_time = (1 - self.tokens) / self.rate
				time.sleep(wait_time)
				self.tokens = 0  # We've used the token that became available
				self.last_update = time.time()
				return True

# Create a global rate limiter
rate_limiter = TokenBucketRateLimiter(rate=3, capacity=3)

def process_batch(batch):
	"""Process a batch of prompts with thread-local SQLite connection"""
	# Create a thread-local database connection
	conn = sqlite3.connect('embeddings.db')
	cursor = conn.cursor()
	
	results = []
	for prompt in batch:
		# Skip empty prompts
		if not prompt:
			continue
			
		# Create a hash of the prompt
		text_hash = hashlib.md5(prompt.encode()).hexdigest()
		
		# Check if embedding exists in database
		cursor.execute("SELECT embedding FROM embeddings WHERE prompt_hash = ?", (text_hash,))
		result = cursor.fetchone()
		
		if result:
			# Increment count for this prompt
			cursor.execute("UPDATE embeddings SET count = count + 1 WHERE prompt_hash = ?", (text_hash,))
			conn.commit()
			# We don't need to return the embedding here since we're just generating them
			results.append(prompt)
		else:
			# Apply rate limiting before API call
			rate_limiter.acquire()
			
			# If not in database, get from API
			try:
				embeddings = embed([prompt])
				if embeddings and len(embeddings) > 0:
					embedding = embeddings[0]
					# Store in database
					cursor.execute(
						"INSERT INTO embeddings (prompt_hash, prompt, embedding, count) VALUES (?, ?, ?, ?)",
						(text_hash, prompt, embedding.embedding.tobytes(), 1)
					)
					conn.commit()
					results.append(prompt)
			except Exception as e:
				print(f"Error embedding prompt: {e}")
				# Sleep a bit longer on error to avoid hammering the API
				time.sleep(1)
	
	conn.close()
	return results

def generate_embeddings(prompts, batch_size=64, max_workers=3):
	"""Generate embeddings for prompts in batches with parallel execution"""
	# Ensure the database table exists using your existing function
	conn = setup_embedding_db()
	conn.close()  # Close the main thread connection
	
	# Split prompts into batches
	batches = [prompts[i:i+batch_size] for i in range(0, len(prompts), batch_size)]
	
	# Create progress bar
	progress_bar = tqdm.tqdm(total=len(prompts), desc="Generating embeddings")
	
	processed = []
	
	# Use ThreadPoolExecutor for I/O-bound operations (API calls)
	with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
		# Submit all batches to the executor
		future_to_batch = {executor.submit(process_batch, batch): batch for batch in batches}
		
		# Process results as they complete
		for future in concurrent.futures.as_completed(future_to_batch):
			batch = future_to_batch[future]
			try:
				batch_result = future.result()
				processed.extend(batch_result)
				progress_bar.update(len(batch))
			except Exception as e:
				print(f"Error processing batch: {e}")
	
	progress_bar.close()
	return processed

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
	cursor.execute("SELECT prompt_hash, prompt, count FROM embeddings ORDER BY RANDOM() LIMIT ?", (n,))
	samples = cursor.fetchall()
	
	# Create a DataFrame for display
	df = pd.DataFrame(samples, columns=['Hash', 'Prompt', 'Count'])
	
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
	cursor.execute("SELECT prompt_hash, prompt, count, embedding FROM embeddings WHERE prompt_hash = ?", (hash_value,))
	result = cursor.fetchone()
	
	if not result:
		print(f"No embedding found with hash: {hash_value}")
		conn.close()
		return
	
	# Unpack the result
	prompt_hash, prompt, count, embedding_bytes = result
	
	# Create a DataFrame for the basic info
	df = pd.DataFrame([{
		'Hash': prompt_hash,
		'Prompt': prompt,
		'Count': count
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
	parser.add_argument("--batch-size", type=int, default=64, help="Batch size for embedding generation")
	parser.add_argument("--workers", type=int, default=3, help="Maximum worker threads")
	parser.add_argument("--max-rps", type=float, default=3.0, help="Maximum requests per second")
	args = parser.parse_args()
	
	# Update rate limiter based on command line argument
	rate_limiter.rate = args.max_rps
	
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
		
		# Run the parallel function
		processed = generate_embeddings(prompts, args.batch_size, args.workers)
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
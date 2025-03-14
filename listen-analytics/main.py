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
from embed import embed, setup_embedding_db

# Global rate limiter
class RateLimiter:
	def __init__(self, max_per_second=3):
		self.max_per_second = max_per_second
		self.last_request_time = 0
		self.lock = threading.Lock()
	
	def wait(self):
		with self.lock:
			current_time = time.time()
			time_since_last = current_time - self.last_request_time
			time_to_wait = max(0, 1/self.max_per_second - time_since_last)
			
			if time_to_wait > 0:
				time.sleep(time_to_wait)
			
			self.last_request_time = time.time()

# Create a global rate limiter
rate_limiter = RateLimiter(max_per_second=3)

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
			rate_limiter.wait()
			
			# If not in database, get from API (using your embed function)
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

if __name__ == "__main__":
	parser = argparse.ArgumentParser(description="Chat analysis and embedding generation")
	parser.add_argument("--generate-embeddings", action="store_true", help="Generate embeddings for all prompts")
	parser.add_argument("--analyze-embeddings", action="store_true", help="Analyze existing embeddings")
	parser.add_argument("--batch-size", type=int, default=64, help="Batch size for embedding generation")
	parser.add_argument("--workers", type=int, default=3, help="Maximum worker threads")
	parser.add_argument("--max-rps", type=float, default=3.0, help="Maximum requests per second")
	args = parser.parse_args()
	
	# Update rate limiter based on command line argument
	rate_limiter.max_per_second = args.max_rps
	
	if args.generate_embeddings:
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
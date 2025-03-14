from analyze import load_chats, cluster_prompts, extract_chat_info
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
import numpy as np
import asyncio
import tqdm
import argparse
from embed import embed, setup_embedding_db

async def generate_embeddings(prompts, batch_size=48, max_concurrency=3):
	"""Generate embeddings for prompts in batches with concurrency control"""
	conn = setup_embedding_db()
	semaphore = asyncio.Semaphore(max_concurrency)
	
	async def process_batch(batch):
		async with semaphore:
			# This would be async in a real implementation
			# For now, we'll just call the synchronous function
			return embed(batch)
	
	# Split prompts into batches
	batches = [prompts[i:i+batch_size] for i in range(0, len(prompts), batch_size)]
	
	# Create progress bar
	progress_bar = tqdm.tqdm(total=len(prompts), desc="Generating embeddings")
	
	# Process batches with concurrency control
	tasks = []
	for batch in batches:
		task = asyncio.create_task(process_batch(batch))
		tasks.append(task)
	
	# Wait for all tasks to complete
	results = []
	for task in asyncio.as_completed(tasks):
		batch_result = await task
		results.extend(batch_result)
		progress_bar.update(len(batch_result))
	
	progress_bar.close()
	return results

if __name__ == "__main__":
	parser = argparse.ArgumentParser(description="Chat analysis and embedding generation")
	parser.add_argument("--generate-embeddings", action="store_true", help="Generate embeddings for all prompts")
	parser.add_argument("--batch-size", type=int, default=48, help="Batch size for embedding generation")
	parser.add_argument("--concurrency", type=int, default=3, help="Maximum concurrent requests")
	args = parser.parse_args()
	
	if args.generate_embeddings:
		print("Loading chats for embedding generation...")
		chats = load_chats()
		prompts = [chat.chat_request.get('prompt') for chat in chats if chat.chat_request.get('prompt')]
		print(f"Found {len(prompts)} prompts for embedding")
		
		# Run the async function in the event loop
		asyncio.run(generate_embeddings(prompts, args.batch_size, args.concurrency))
		print("Embedding generation complete")
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
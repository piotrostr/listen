from analyze import load_chats, extract_chat_info, cluster_prompts
import pandas as pd
from sklearn.feature_extraction.text import TfidfVectorizer
import numpy as np

if __name__ == "__main__":
	# chats = load_chats()
	# chat_data = [extract_chat_info(chat) for chat in chats]
	# df = pd.DataFrame(chat_data)
	# df.head()
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
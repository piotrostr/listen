import json
from analyze import extract_chat_info, load_chats, Chat

def test_load_chats():
	load_chats(True)

def test_extract_chat_info():
	chat = load_chats(True)[0]
	chat_info = extract_chat_info(chat)
	print(chat_info.keys())

def test_embed():
	from embed import embed
	prompts = ["What is the capital of France?", "What is the capital of Germany?"]
	embeddings = embed(prompts)
	assert len(embeddings) == 2
	assert embeddings[0].embedding.shape == (1024,)
	assert embeddings[1].embedding.shape == (1024,)


def test_redis():
	import redis
	redis_client = redis.Redis(host='localhost', port=6379, db=0)
	redis_client.set('test', 'test')
	assert redis_client.get('test') == b'test'
	redis_client.delete('test')
	assert redis_client.get('test') is None
	
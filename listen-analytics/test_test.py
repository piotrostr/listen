import json
from analyze import extract_chat_info, load_chats, Chat

def test_load_chats():
	load_chats(True)

def test_extract_chat_info():
	chat = load_chats(True)[0]
	chat_info = extract_chat_info(chat)
	print(chat_info.keys())

def test_embed():
	from analyze import embed
	prompts = ["What is the capital of France?", "What is the capital of Germany?"]
	embeddings = embed(prompts)
	assert len(embeddings) == 2
	assert embeddings[0].embedding.shape == (3072,)
	assert embeddings[1].embedding.shape == (3072,)

test_embed()
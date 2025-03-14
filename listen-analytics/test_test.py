import json
from analyze import extract_chat_info, load_chats, Chat

def test_load_chats():
	load_chats(True)

def test_extract_chat_info():
	chat = load_chats(True)[0]
	chat_info = extract_chat_info(chat)
	print(chat_info.keys())
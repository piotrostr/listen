from analyze import load_chats, extract_chat_info
import pandas as pd

if __name__ == "__main__":
	# chats = load_chats()
	# chat_data = [extract_chat_info(chat) for chat in chats]
	# df = pd.DataFrame(chat_data)
	# df.head()
	chats = load_chats(True)
	prompts = [chat.chat_request['prompt'] for chat in chats]
	print(prompts)
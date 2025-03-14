import pandas as pd
import json
import glob
import numpy as np
from pprint import pprint

import pydantic
from typing import Optional, List, Dict, Any

class Chat(pydantic.BaseModel):
    _id: Optional[str] = None
    user_id: Optional[str] = None
    wallet_address: Optional[str] = None
    pubkey: Optional[str] = None
    chat_request: Dict[str, Any]
    responses: List[Dict[str, Any]]

def load_chats(one_file: bool = False) -> List[Chat]:
    """Load all chat files from raw_chats directory and parse them into Chat objects."""
    all_chats = []
    
    for file_path in glob.glob("raw_chats/*.json"):
        try:
            with open(file_path, "r") as f:
                chats_chunk = json.load(f)
                
                # Handle both list and dict formats
                if isinstance(chats_chunk, dict):
                    chats_chunk = [chats_chunk]
                
                for chat_data in chats_chunk:
                    try:
                        chat = Chat(**chat_data)
                        all_chats.append(chat)
                    except Exception as e:
                        print(f"Error parsing chat: {e}")
        except Exception as e:
            print(f"Error loading file {file_path}: {e}")
        if one_file:
            break
    
    print(f"Loaded {len(all_chats)} chats successfully")
    return all_chats

def extract_chat_info(chat: Chat) -> Dict[str, Any]:
    """Extract relevant information from a Chat object."""
    # Basic info
    info = {
        'chat_id': chat._id,
        'user_id': chat.user_id,
        'wallet_address': chat.wallet_address,
        'pubkey': chat.pubkey,
    }
    
    # Extract data from chat_request
    if chat.chat_request:
        # Extract prompt
        info['prompt'] = chat.chat_request.get('prompt')
        
        # Extract chain information
        info['chain'] = chat.chat_request.get('chain')
        
        # Extract chat history
        chat_history = chat.chat_request.get('chat_history', [])
        info['chat_history_length'] = len(chat_history)
        
        # Extract user messages from chat history
        user_messages = [
            msg.get('content', []) 
            for msg in chat_history 
            if msg.get('role') == 'user'
        ]
        info['user_message_count'] = len(user_messages)
        
        # Extract preamble if available
        info['has_preamble'] = 'preamble' in chat.chat_request
    
    # Extract response content
    if chat.responses and len(chat.responses) > 0:
        info['response_content'] = chat.responses[0].get('content')
        info['response_type'] = chat.responses[0].get('type')
        info['response_count'] = len(chat.responses)
    
    return info

def create_chat_dataframe(chats: List[Chat]) -> pd.DataFrame:
    """Create a pandas DataFrame from a list of Chat objects."""
    chat_info_list = [extract_chat_info(chat) for chat in chats]
    return pd.DataFrame(chat_info_list)

def analyze_chats():
    """Main function to load, process and analyze chats."""
    chats = load_chats()
    
    if not chats:
        print("No chats loaded. Check the raw_chats directory.")
        return
    
    # Create DataFrame for analysis
    df = create_chat_dataframe(chats)
    
    # Print basic statistics
    print(f"\nTotal chats: {len(df)}")
    print(f"Unique users: {df['user_id'].nunique()}")
    
    if 'chain' in df.columns:
        print("\nChain distribution:")
        print(df['chain'].value_counts())
    
    # You can add more analysis here
    
    return df

if __name__ == "__main__":
    df = analyze_chats()
    # Optional: save to CSV
    # df.to_csv("chat_analysis.csv", index=False)

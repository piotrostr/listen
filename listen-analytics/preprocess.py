import re
def preprocess_text(text):
    """Enhanced crypto-aware preprocessing"""
    text = text.lower().strip()
    
    # Preserve crypto-specific patterns
    text = re.sub(r'\$[a-z]+', '[TOKEN]', text)  # Normalize token names
    text = re.sub(r'0x[a-f0-9]+', '[CONTRACT]', text)  # Contract addresses
    text = re.sub(r'\b[a-z0-9]{40,}\b', '[HASH]', text)  # Long hashes
    
    # Remove non-essential punctuation but keep question marks
    text = re.sub(r'[^\w\s$.-?]', '', text)
    return text
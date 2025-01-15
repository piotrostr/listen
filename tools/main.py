tools = [
    {
        "name": "get_balance",
        "description": "Get the SOL balance for a Solana wallet address. Returns the balance in lamports (1 SOL = 1,000,000,000 lamports).",
        "input_schema": {
            "type": "object",
            "properties": {
                "pubkey": {
                    "type": "string",
                    "description": "The Solana wallet public key/address",
                }
            },
            "required": ["pubkey"],
        },
    },
    {
        "name": "get_token_balance",
        "description": "Get the token balance for a specific SPL token in a Solana wallet. Returns the raw token amount without decimals.",
        "input_schema": {
            "type": "object",
            "properties": {
                "pubkey": {
                    "type": "string",
                    "description": "The Solana wallet public key/address",
                },
                "mint": {"type": "string", "description": "The SPL token mint address"},
            },
            "required": ["pubkey", "mint"],
        },
    },
    {
        "name": "get_token_price",
        "description": "Get the current price of an SPL token in USD.",
        "input_schema": {
            "type": "object",
            "properties": {
                "mint": {"type": "string", "description": "The SPL token mint address"}
            },
            "required": ["mint"],
        },
    },
    {
        "name": "swap_tokens",
        "description": "Perform a token swap on Solana between two SPL tokens. The amount is the input amount in raw token units. Slippage is in basis points (1 bp = 0.01%).",
        "input_schema": {
            "type": "object",
            "properties": {
                "input_mint": {
                    "type": "string",
                    "description": "The input token mint address",
                },
                "output_mint": {
                    "type": "string",
                    "description": "The output token mint address",
                },
                "amount": {
                    "type": "integer",
                    "description": "The input amount in raw token units",
                    "minimum": 0,
                },
                "slippage": {
                    "type": "integer",
                    "description": "Maximum allowed slippage in basis points (1 bp = 0.01%)",
                    "minimum": 0,
                },
            },
            "required": ["input_mint", "output_mint", "amount", "slippage"],
        },
    },
    {
        "name": "get_wallet_holdings",
        "description": "Get all token holdings for a Solana wallet, including the token mint addresses, associated token accounts, and amounts.",
        "input_schema": {"type": "object", "properties": {}, "required": []},
    },
]

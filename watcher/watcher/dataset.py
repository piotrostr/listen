import os

import pandas as pd

from watcher.constants import FUCK_ADDRESS
from watcher.types import Holding
from watcher.watcher import Chain, Watcher

TOKENS_INVESTED_PATH = "./tokens_invested_all.csv"
CANDLES_PATH = "./candles.csv"


def get_or_create_tokens_invested_df(update=False):
    if os.path.exists(TOKENS_INVESTED_PATH) and not update:
        return pd.read_csv(TOKENS_INVESTED_PATH)
    watcher = Watcher(Chain.SOL)
    all_holdings: list[Holding] = []
    tokens_invested_response = watcher.grab_tokens_invested(FUCK_ADDRESS)
    all_holdings.extend(tokens_invested_response.data.holdings)
    next_page_token = tokens_invested_response.data.next
    while next_page_token:
        tokens_invested_response = watcher.grab_tokens_invested(
            FUCK_ADDRESS,
            next_page_token=next_page_token,
        )
        next_page_token = tokens_invested_response.data.next
        all_holdings.extend(tokens_invested_response.data.holdings)

    df = pd.DataFrame([vars(holding) for holding in all_holdings])
    df.to_csv("tokens_invested_all.csv")
    return df


def get_or_create_candles_df(update=False):
    """
    get_or_create_candles_df fetches the candles for all of the investments in
    the tokens_invested_df

    it might make sense to implement retries and hash map (address => bool) done
    as there might be errors/rate limits (non-public API)
    """
    if os.path.exists(CANDLES_PATH) and not update:
        return pd.read_csv(CANDLES_PATH)
    watcher = Watcher(chain=Chain.SOL)
    tokens_invested_df = get_or_create_tokens_invested_df()
    token_addresses = tokens_invested_df["token_address"].tolist()
    timestamps = tokens_invested_df["last_active_timestamp"].tolist()

    candles = []

    # Step 4: Fetch data for each token
    for timestamp, token_address in zip(timestamps, token_addresses):
        print(f"Fetching data for token: {token_address}")

        # Fetch time series data for the token
        candles_response = watcher.get_first_1k_candles(token_address, timestamp, "1m")
        time_series_df = pd.DataFrame(
            [vars(candles) for candles in candles_response.data]
        )
        candles.append(time_series_df)

    # Step 5: Combine all data into a single DataFrame
    combined_candles = pd.concat(candles, ignore_index=True)

    # Step 6: Save the DataFrame to a CSV file
    combined_candles.to_csv(CANDLES_PATH, index=False)

    print(
        f"Dataset saved to {CANDLES_PATH} with {
          len(combined_candles)} rows."
    )

    return combined_candles

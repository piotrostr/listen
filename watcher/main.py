import logging

from watcher.dataset import get_or_create_candles_df, get_or_create_tokens_invested_df
from watcher.types import Chain, Timeframe
from watcher.watcher import Watcher

log_format = "%(asctime)s - %(levelname)s - %(message)s"
logging.basicConfig(level=logging.INFO, format=log_format)


def make_data():
    get_or_create_tokens_invested_df()
    get_or_create_candles_df(update=True)


if __name__ == "__main__":
    import time

    watcher = Watcher(Chain.SOL)
    watcher.get_first_1k_candles(
        "Fosp9yoXQBdx8YqyURZePYzgpCnxp9XsfnQq69DRvvU4",
        last_active_timestamp=int(time.time()),
        timeframe=Timeframe.FIFTEEN_MINUTES,
    )

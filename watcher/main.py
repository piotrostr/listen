import logging

from watcher.dataset import get_or_create_candles_df, get_or_create_tokens_invested_df

log_format = "%(asctime)s - %(levelname)s - %(message)s"
logging.basicConfig(level=logging.INFO, format=log_format)


def make_data():
    get_or_create_tokens_invested_df()
    get_or_create_candles_df(update=True)


if __name__ == "__main__":
    pass

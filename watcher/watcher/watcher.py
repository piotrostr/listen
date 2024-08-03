import logging
import time

import requests

from .types import Chain, GetCandlesResponse, GrabTokensInvestedResponse, Timeframe


class Watcher:
    base = "https://gmgn.ai/defi/quotation/v1"
    session: requests.Session
    chain: Chain

    def __init__(self, chain: Chain):
        self.chain = chain
        self.session = requests.Session()

        self.clines_path = f"/tokens/kline/{self.chain.value}/"
        self.holdings_path = f"/wallet/{self.chain.value}/holdings/"

    def get_candles(
        self, token_address: str, start: int, end: int, timeframe: str
    ) -> GetCandlesResponse:
        url = self.base + self.clines_path + token_address
        res = self.session.get(
            url,
            params={
                "resolution": timeframe,
                "from": start,
                "to": end,
            },
        )
        res.raise_for_status()
        # print(res.json())
        logging.debug(res.url)
        return GetCandlesResponse.model_validate(res.json())

    def get_first_1k_candles(
        self, token_address: str, last_active_timestamp: int, timeframe: Timeframe
    ) -> GetCandlesResponse:
        # print as human readable
        logging.debug(
            time.strftime(
                "last active: %Y-%m-%d %H:%M:%S", time.localtime(last_active_timestamp)
            )
        )
        candles = self.get_candles(
            token_address, 0, last_active_timestamp, timeframe.value
        )
        if not candles.data:
            raise ValueError("No candles found for token", token_address)
        first_candle_timestamp = min([c.time for c in candles.data])
        # convert to seconds
        first_candle_timestamp = first_candle_timestamp // 1000
        logging.debug(
            time.strftime(
                "first candle: %Y-%m-%d %H:%M:%S",
                time.localtime(first_candle_timestamp),
            )
        )
        return self.get_candles(
            token_address,
            first_candle_timestamp - 10,
            first_candle_timestamp + timeframe.in_seconds() * 1000,
            timeframe.value,
        )

    def grab_tokens_invested(
        self, wallet_address: str, next_page_token: str | None = None
    ) -> GrabTokensInvestedResponse:
        """grab_tokens_invested returns a JSON object and this is intended, no
        need for typings just now since it is for pandas analysis
        """
        url = self.base + self.holdings_path + wallet_address
        params = {
            "orderby": "last_active_timestamp",
            "direction": "desc",
            "showsmall": "true",
            "sellout": "false",
            "limit": "100",
        }
        if next_page_token is not None:
            params["cursor"] = next_page_token
        res = self.session.get(url, params=params)
        res.raise_for_status()
        response = GrabTokensInvestedResponse.model_validate(res.json())
        return response


def last_50_candles(timeframe: Timeframe) -> tuple[int, int]:
    """
    last_50_candles actually returns 51, but it's OK
    """
    now = time.time()
    return timeframe.in_seconds() * 50, int(now)

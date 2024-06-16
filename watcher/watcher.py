import time
from enum import Enum

import pydantic
import requests


class Chain(Enum):
    SOL = "sol"


class Candle(pydantic.BaseModel):
    open: float
    high: float
    low: float
    close: float
    volume: float
    time: int


class GenericResponse(pydantic.BaseModel):
    code: int  # TODO this should be an enum and handled properly
    msg: str


class GetCandlesResponse(GenericResponse):
    code: int
    msg: str
    data: list[Candle]


class GrabTokensInvestedResponse(GenericResponse):
    data: object


class Watcher:
    base = "https://gmgn.ai/defi/quotation/v1"
    session: requests.Session
    chain: Chain

    def __init__(self, chain: Chain):
        self.chain = chain
        self.session = requests.Session()

        self.clines_path = f"/tokens/kline/{self.chain.value}/"
        self.holdings_path = f"/wallet/{self.chain.value}/holdings/"

    def get_candles(self, token_address, start, end, timeframe) -> GetCandlesResponse:
        url = self.base + self.clines_path + token_address
        print(url)
        res = self.session.get(
            url,
            params={
                "resolution": timeframe,
                "from": start,
                "to": end,
            })
        res.raise_for_status()
        return GetCandlesResponse.model_validate(res.json())

    def grab_tokens_invested(self, wallet_address: str) -> GrabTokensInvestedResponse:
        """grab_tokens_invested returns a JSON object and this is intended, no
        need for typings just now since it is for pandas analysis
        """
        url = self.base + self.holdings_path + wallet_address
        res = self.session.get(url, params={
            "orderby": "last_active_timestamp",
            "direction": "desc",
            "showsmall": "true",
            "sellout": "false"
        })
        res.raise_for_status()
        return GrabTokensInvestedResponse.model_validate(res.json())


class Timeframe(Enum):
    ONE_MINUTE = "1m"
    FIVE_MINUTES = "5m"
    FIFTEEN_MINUTES = "15m"
    THIRTY_MINUTES = "30m"
    ONE_HOUR = "1h"
    FOUR_HOURS = "4h"
    ONE_DAY = "1d"


def last_50_candles(timeframe: Timeframe) -> tuple[int, int]:
    """
    last_50_candles actually returns 51, but it's OK
    """
    now = time.time()
    match timeframe:
        case Timeframe.ONE_MINUTE:
            return now - 60 * 50, now
        case Timeframe.FIVE_MINUTES:
            return now - 60 * 5 * 50, now
        case Timeframe.FIFTEEN_MINUTES:
            return now - 60 * 15 * 50, now
        case Timeframe.THIRTY_MINUTES:
            return now - 60 * 30 * 50, now
        case Timeframe.ONE_HOUR:
            return now - 60 * 60 * 50, now
        case Timeframe.FOUR_HOURS:
            return now - 60 * 60 * 4 * 50, now
        case Timeframe.ONE_DAY:
            return now - 60 * 60 * 24 * 50, now
        case _:
            raise ValueError("Invalid timeframe")

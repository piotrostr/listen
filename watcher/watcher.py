import time
from enum import Enum

import pydantic
import requests

BOME = "ukHH6c7mMyiWCf1b9pnWe25TSpkDDt3H5pQZgZ74J82"


class Chain(Enum):
    SOL = "sol"


class Candle(pydantic.BaseModel):
    open: float
    high: float
    low: float
    close: float
    volume: float
    time: int


class GetCandlesResponse(pydantic.BaseModel):
    code: int  # TODO this should be an enum and handled properly
    msg: str
    data: list[Candle]


class Watcher:
    base = "https://gmgn.ai"
    clines_route = f"/defi/quotation/v1/tokens/kline"
    session: requests.Session
    chain: Chain

    def __init__(self, chain: Chain):
        self.chain = chain
        self.session = requests.Session()

    def get_candles(self, token_address, start, end, timeframe) -> GetCandlesResponse:
        url = f"{self.base}{
            self.clines_route}/{self.chain.value}/{token_address}"
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

    def grab_tokens_invested(self):
        pass


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

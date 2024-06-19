from enum import Enum
from typing import Optional

from pydantic import BaseModel


class Timeframe(Enum):
    ONE_MINUTE = "1m"
    FIVE_MINUTES = "5m"
    FIFTEEN_MINUTES = "15m"
    THIRTY_MINUTES = "30m"
    ONE_HOUR = "1h"
    FOUR_HOURS = "4h"
    ONE_DAY = "1d"

    def in_seconds(self) -> int:
        match self:
            case Timeframe.ONE_MINUTE:
                return 60
            case Timeframe.FIVE_MINUTES:
                return 60 * 5
            case Timeframe.FIFTEEN_MINUTES:
                return 60 * 15
            case Timeframe.THIRTY_MINUTES:
                return 60 * 30
            case Timeframe.ONE_HOUR:
                return 60 * 60
            case Timeframe.FOUR_HOURS:
                return 60 * 60 * 4
            case Timeframe.ONE_DAY:
                return 60 * 60 * 24


class Chain(Enum):
    SOL = "sol"


class Candle(BaseModel):
    open: float
    high: float
    low: float
    close: float
    volume: float
    time: int


class Holding(BaseModel):
    address: str
    token_address: str
    symbol: str
    name: str
    decimals: int
    logo: Optional[str]
    balance: str
    usd_value: float
    realized_profit_30d: float
    realized_profit: float
    realized_pnl: Optional[float]
    realized_pnl_30d: float
    unrealized_profit: float
    unrealized_pnl: Optional[float]
    total_profit: float
    total_profit_pnl: float
    avg_cost: float
    avg_sold: Optional[float]
    buy_30d: int
    sell_30d: int
    sells: int
    price: float
    cost: float
    position_percent: float
    last_active_timestamp: int
    history_sold_income: float
    history_bought_cost: float
    price_change_5m: float
    price_change_1h: float
    price_change_6h: float
    price_change_24h: float
    is_following: bool
    is_show_alert: bool
    is_honeypot: Optional[bool]


class GenericResponse(BaseModel):
    code: int  # TODO this should be an enum and handled properly
    msg: str


class GetCandlesResponse(GenericResponse):
    code: int
    msg: str
    data: list[Candle]


class GrabTokensInvestedResponse(GenericResponse):
    class Data(BaseModel):
        holdings: list[Holding]
        next: Optional[str]

    data: Data

from watcher.constants import BOME, FUCK_ADDRESS
from watcher.types import Chain, Timeframe
from watcher.watcher import Watcher, last_50_candles


def test_get_candles():
    watcher = Watcher(Chain.SOL)
    timeframe = Timeframe.ONE_MINUTE
    start, end = last_50_candles(timeframe)
    candles = watcher.get_candles(
        token_address=BOME, start=start, end=end, timeframe=timeframe.value
    )
    assert candles.code == 0
    assert candles.msg == "success"
    assert 55 > len(candles.data) > 50


def test_grab_tokens_invested():
    watcher = Watcher(Chain.SOL)
    tokens_response = watcher.grab_tokens_invested(FUCK_ADDRESS)
    assert len(tokens_response.data.holdings) > 0


def test_get_first_1k_candles():
    watcher = Watcher(Chain.SOL)
    token_address = "52ScRbUR7y8AVqkMjE29FJVjDPELSPofJwaXaLHepump"
    last_active_timestamp = 1718716069
    candles_response = watcher.get_first_1k_candles(
        token_address, last_active_timestamp, Timeframe.ONE_MINUTE
    )
    assert 1005 > len(candles_response.data) > 1000
    import pandas as pd

    from watcher.plot import make_fig

    df = pd.DataFrame([vars(candle) for candle in candles_response.data])
    fig = make_fig(df)

    fig.show()

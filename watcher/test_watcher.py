from watcher.constants import BOME, FUCK_ADDRESS
from watcher.watcher import Chain, Timeframe, Watcher, last_50_candles


def test_get_candles():
    watcher = Watcher(Chain.SOL)
    timeframe = Timeframe.ONE_MINUTE
    start, end = last_50_candles(timeframe)
    candles = watcher.get_candles(
        token_address=BOME,
        start=start,
        end=end,
        timeframe=timeframe.value
    )
    assert candles.code == 0
    assert candles.msg == "success"
    assert 55 > len(candles.data) > 50


def test_grab_tokens_invested():
    watcher = Watcher(Chain.SOL)
    tokens = watcher.grab_tokens_invested(FUCK_ADDRESS)
    assert len(tokens) > 0

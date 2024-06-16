from watcher import BOME, Chain, Timeframe, Watcher, last_50_candles


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

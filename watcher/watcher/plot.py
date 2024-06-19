import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots


def time_unix_ms_to_str(time: int) -> str:
    return pd.to_datetime(time, unit="ms").strftime("%Y-%m-%d %H:%M:%S")


def make_fig(df: pd.DataFrame) -> go.Figure:
    fig = make_subplots(
        rows=2,
        cols=1,
        shared_xaxes=True,
        vertical_spacing=0.03,
        subplot_titles=("OHLC", "Volume"),
        row_width=[0.2, 0.7],
    )
    fig.add_trace(
        go.Candlestick(
            x=df["time"].map(time_unix_ms_to_str),
            open=df["open"],
            high=df["high"],
            low=df["low"],
            close=df["close"],
        ),
        row=1,
        col=1,
    )
    fig.add_trace(
        go.Bar(
            x=df["time"].map(time_unix_ms_to_str),
            y=df["volume"],
            showlegend=False,
        ),
        row=2,
        col=1,
    )
    fig.update(layout_xaxis_rangeslider_visible=False)
    return fig

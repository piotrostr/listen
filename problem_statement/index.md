# Finding the dips

## Context

- there are tokens from community platform pump.fun on Solana, which after
  reaching a certain marketcap (over 62k USD) are listed on Raydium

- some symbols reach a market cap of up to 100M USD, yielding great returns

## Problem

- there are probably about 200 launches of such tokens and buying the token
  blindly sometimes works, but a lot of the tokens go -90% when they hit the
  Raydium exchange

  1. sniping on pump.fun makes no sense since there are roughly 15k tokens
     launched daily, mostly programatically through bots
  2. sniping on raydium is essentially top-blasting, it is also unlikely to be
     profitable in a large sample of tokens, giving the poor entry and 0.3%
     maybe reaching high market capitalization

## Solution: Dips!

- my goal is to train an ML model (could be forecasting, AutoML from BigQuery,
  anything decent really for fitting to check for the "dip" moment)

- the data will be in a form of a constant stream for any given token, the
  model could even be a binary classifier that marks the "dip"

prompts:

- my first question would be what would be the best way to detect the dip, on the chart, it is a period when the price has corrected, sometimes between -50% to -90% from the top and the volume has relaxed, then a slow reaccumulation happens and volume starts increasing gradually, this is the moment i want to spot

## Starters

- will try fibonacci retracement of 0.782 here, since the data source is the
  crucial part

- a mock socket that could stream the trading data for each of the blocks could
  be nice, retrieving first 2 hours of transactions not so much

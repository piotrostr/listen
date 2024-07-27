import { TradeCreatedSchema } from "./trade";
import { NewCoinCreatedSchema } from "./new-coin";

enum MessageType {
  TRADE_CREATED = `42["tradeCreated"`,
  COIN_CREATED = `42["newCoinCreated"`,
}

interface PumpBuyRequest {
  mint: string;
}

async function main() {
  let pumpUrl =
    "wss://frontend-api.pump.fun/socket.io/?EIO=4&transport=websocket";
  let ws = new WebSocket(pumpUrl);

  ws.onopen = function () {
    console.log("Connection established");
    ws.send("40");
  };

  ws.onmessage = function (event) {
    let data = event.data as string;
    if (data == "2") {
      ws.send("3");
      console.log("Heartbeat sent");
    } else if (data.startsWith(MessageType.TRADE_CREATED)) {
      return;
      let jsonParsable = data
        .replace(`42["tradeCreated",`, "")
        .replace("]", "");
      let trade = TradeCreatedSchema.parse(JSON.parse(jsonParsable));
      console.log({
        signature: trade.signature,
        sol_amount: trade.sol_amount,
        token_amount: trade.token_amount,
        is_buy: trade.is_buy,
        timestamp: trade.timestamp,
        name: trade.name,
        symbol: trade.symbol,
        usd_market_cap: trade.usd_market_cap,
      });
    } else if (data.startsWith(MessageType.COIN_CREATED)) {
      let jsonParsable = data
        .replace(`42["newCoinCreated",`, "")
        .replace("]", "");
      let coin = NewCoinCreatedSchema.parse(JSON.parse(jsonParsable));
      console.log({
        mint: coin.mint,
        at: new Date(coin.created_timestamp).toLocaleString(),
      });
    } else {
      console.log(data);
    }
  };

  ws.onclose = function () {
    console.log("Connection closed");
  };

  ws.onerror = function (event) {
    console.log("Error: " + event);
  };
}

await main();

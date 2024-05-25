package main

import (
	"log"
	"os"
	"os/signal"
	"time"

	"github.com/gorilla/websocket"
)

func main() {
	log.SetFlags(0)
	interrupt := make(chan os.Signal, 1)
	signal.Notify(interrupt, os.Interrupt)

	c, _, err := websocket.DefaultDialer.Dial(os.Getenv("WS_URL"), nil)
	if err != nil {
		panic(err)
	}
	defer c.Close()

	payload := `{
		"jsonrpc": "2.0",
		"id": 420,
		"method": "transactionSubscribe",
		"params": [
			{
				"vote": false,
				"failed": false,
				"signature": null,
				"accountRequired": ["7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5"]
			},
			{
				"commitment": "processed",
				"encoding": "base64",
				"transaction_details": "signatures",
				"showRewards": true,
				"maxSupportedTransactionVersion": 0
			}
		]
	}`

	done := make(chan struct{})

	go func() {
		defer close(done)
		for {
			_, message, err := c.ReadMessage()
			if err != nil {
				panic(err)
			}
			log.Printf("recv: %s", message)
		}
	}()

	err = c.WriteMessage(websocket.TextMessage, []byte(payload))
	if err != nil {
		panic(err)
	}

	for {
		select {
		case <-done:
			return
		case <-interrupt:
			log.Println("interrupt")

			// Cleanly close the connection by sending a close message and then
			// waiting (with timeout) for the server to close the connection.
			err := c.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
			if err != nil {
				log.Println("write close:", err)
				return
			}
			select {
			case <-done:
			case <-time.After(time.Second):
			}
			return
		}
	}
}

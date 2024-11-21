import dotenv
import os
from typing import List
from pydantic import BaseModel
from datetime import datetime
import requests
import json


# Response Models
class DebugData(BaseModel):
    poolAddress: str


class ChartResponse(BaseModel):
    t: List[int]  # timestamps
    o: List[float]  # open prices
    h: List[float]  # high prices
    l: List[float]  # low prices
    c: List[float]  # close prices
    v: List[float]  # volumes
    debugData: DebugData


class BullXClient:
    def __init__(self, auth_token: str):
        self.base_url = "https://api-edge.bullx.io"
        self.auth_token = auth_token

    def get_headers(self):
        return {
            "accept": "application/json, text/plain, */*",
            "accept-language": "en-US,en;q=0.9",
            "authorization": f"Bearer {self.auth_token}",
            "content-type": "text/plain",
            "origin": "https://bullx.io",
            "referer": "https://bullx.io/",
            "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        }

    def get_chart_data(
        self,
        chain_id: int,
        base: str,
        quote: str,
        from_timestamp: int,
        to_timestamp: int,
        interval_secs: int = 5 * 60,
        count_back: int = 255,
    ) -> ChartResponse:
        # Create the request data as a string
        request_data = {
            "name": "chart",
            "data": {
                "chainId": chain_id,
                "base": base,
                "quote": quote,
                "from": from_timestamp - 24 * 60 * 60 * 1000,  # Subtract 24 hours
                "to": to_timestamp,
                "intervalSecs": interval_secs,
                "countBack": count_back,
            },
        }

        # Convert to JSON string
        request_data_str = json.dumps(request_data)

        response = requests.post(
            f"{self.base_url}/chart",
            headers=self.get_headers(),
            data=request_data_str,  # Send as raw string data
        )

        try:
            response.raise_for_status()
            return ChartResponse.model_validate(response.json())
        except requests.exceptions.HTTPError as e:
            print(f"Error response: {e.response.text}")
            raise


# Usage example:
if __name__ == "__main__":
    dotenv.load_dotenv()

    AUTH_TOKEN = os.environ.get("AUTH_TOKEN")
    if not AUTH_TOKEN:
        raise ValueError("Please set the AUTH_TOKEN environment variable")

    FROM = 1731801600
    TO = 1732146799

    client = BullXClient(AUTH_TOKEN)

    with open("./graduates.json", "r") as f:
        data = json.load(f)

    pubkeys = [i["pubkey"] for i in data]

    responses = []

    for pubkey in pubkeys[:5]:
        response = client.get_chart_data(
            chain_id=1399811149,
            base=pubkey,
            quote="So11111111111111111111111111111111111111112",
            from_timestamp=FROM,
            to_timestamp=TO,
        )
        # Access the data
        print(f"Number of candles: {len(response.t)}")
        print(f"First timestamp: {datetime.fromtimestamp(response.t[0]/1000)}")
        print(f"Pool Address: {response.debugData.poolAddress}")
        responses.append(response)

    with open("candles.json", "w") as f:
        f.write(json.dumps([r.dict() for r in responses], indent=2))

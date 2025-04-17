import json
import argparse
import os
import asyncio

from zep_cloud.client import AsyncZep
import dotenv
from graphiti import make_graphiti, read_in_sample_tool_outputs, insert, query

dotenv.load_dotenv()

group_id = "listen-memory"


async def insert(zep_client: AsyncZep):
    tool_outputs = read_in_sample_tool_outputs()
    for tool_output in tool_outputs:
        await zep_client.graph.add(
            type="json",
            data=json.dumps(tool_output),
            group_id=group_id,
        )


async def query(zep_client: AsyncZep, query: str):
    response = await zep_client.graph.search(
        query=query,
        group_id=group_id,
    )
    print(response)


async def main(args):
    zep_client = AsyncZep(
        api_key=os.environ["ZEP_API_KEY"],
    )
    # graphiti = make_graphiti()

    if args.insert:
        await insert(zep_client)

    if args.query:
        await query(zep_client, args.query)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--insert", action="store_true")
    parser.add_argument("--query", type=str)
    args = parser.parse_args()
    asyncio.run(main(args))

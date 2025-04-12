import httpx
import pytest
from memory import make_memory

@pytest.mark.skip(reason="server test requires running server")
async def test_server():
    async with httpx.AsyncClient() as client:
        response = client.get("http://localhost:9696/health")
        assert response.json() == {"status": "ok"}

        response = await client.post(
            "http://localhost:9696/memories",
            json={
                "messages": [
                    {
                        "role": "user",
                        "content": 'Result of tool call fetch_price_action_analysis with params: {"mint":"Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump","intent":"get a feel for the chart","interval":"15m"}: "Okay, here\'s a brief analysis of the candlestick data:\n\n*   **Overall Trend:**  The chart shows an overall upward trend.\n\n*   **Recent Price Action:** There was a strong bullish spike near timestamp 1744437600 with high volume, price increase of ~24% from 0.00277 to 0.00341. Before that there was a period of sideways movement and choppy price action. After the spike, the price continues upwards with some retracements.\n\n*   **Volatility:**  Volatility appears to have increased significantly after the price spike around timestamp 1744437600.\n\n*   **Reversal/Continuation Signals:** The strong bullish move suggests a potential continuation of the upward trend, but the increased volatility indicates a higher risk of reversals.\n\nIn summary, the chart suggests a bullish trend, with a recent major price spike indicating significant buying pressure. However, increased volatility means potential for sharp reversals.\n"',
                    }
                ],
                "config": {},
            },
        )
        data = response.json()
        print(data)

def test_e2e():
    memory = make_memory()
    agent_id = "john"
    
    # Add a single memory
    print("\nAdding a single memory...")
    result1 = memory.add(
        "Hi, my name is John and I am a software engineer.",
        None,
        agent_id
    )
    print("Added memory:", result1)

    # Add multiple messages
    print("\nAdding multiple messages...")
    result2 = memory.add(
        [
            {"role": "user", "content": "What is your favorite city?"},
            {"role": "assistant", "content": "I love Paris, it is my favorite city."}
        ],
        None,
        agent_id
    )
    print("Added messages:", result2)

    # Trying to update the memory
    result3 = memory.add(
        [
            {"role": "user", "content": "What is your favorite city?"},
            {"role": "assistant", "content": "I love New York, it is my favorite city."}
        ],
        None,
        agent_id
    )
    print("Updated messages:", result3)

    # Get a single memory
    print("\nGetting a single memory...")
    if result1.get("results") and len(result1["results"]) > 0:
        single_memory = memory.get(result1["results"][0]["id"])
        print("Single memory:", single_memory)
    else:
        print("No memory was added in the first step")

    # Get all memories before update
    print("\nGetting all memories before update...")
    all_memories_before = memory.get_all(None, agent_id)
    print("All memories before update:", all_memories_before)

    # Updating memory if we have any
    if all_memories_before.get("results") and len(all_memories_before["results"]) > 0:
        result4 = memory.update(
            all_memories_before["results"][0]["id"],
            "I love India, it is my favorite country."
        )
        print("Updated memory:", result4)
    else:
        print("No memories found to update")

    # Get all memories
    print("\nGetting all memories...")
    all_memories = memory.get_all(None, agent_id)
    print("All memories:", all_memories)

    # Search for memories
    print("\nSearching memories...")
    search_result = memory.search(
        "What do you know about Paris?",
        None,
        agent_id
    )
    print("Search results:", search_result)

    # Get memory history
    if result1.get("results") and len(result1["results"]) > 0:
        print("\nGetting memory history...")
        history = memory.history(result1["results"][0]["id"], None, agent_id)
        print("Memory history:", history)

    # Delete a memory
    if result1.get("results") and len(result1["results"]) > 0:
        print("\nDeleting a memory...")
        memory.delete(result1["results"][0]["id"], None, agent_id)
        print("Memory deleted successfully")

    # Reset all memories
    print("\nResetting all memories...")
    memory.reset()
    print("All memories reset") 
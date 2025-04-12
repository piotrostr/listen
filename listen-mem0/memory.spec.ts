import { describe, expect, test } from "bun:test";
import { makeMemory } from "./memory";

describe("Memory", () => {
  test("server", async () => {
    let isOk = await fetch("http://localhost:9696/health").then((res) =>
      res.json()
    );
    expect(isOk).toEqual({ status: "ok" });

    let res = await fetch("http://localhost:9696/memories", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        messages: [
          {
            role: "user",
            content:
              'Result of tool call fetch_price_action_analysis with params: {"mint":"Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump","intent":"get a feel for the chart","interval":"15m"}: "Okay, here\'s a brief analysis of the candlestick data:\n\n*   **Overall Trend:**  The chart shows an overall upward trend.\n\n*   **Recent Price Action:** There was a strong bullish spike near timestamp 1744437600 with high volume, price increase of ~24% from 0.00277 to 0.00341. Before that there was a period of sideways movement and choppy price action. After the spike, the price continues upwards with some retracements.\n\n*   **Volatility:**  Volatility appears to have increased significantly after the price spike around timestamp 1744437600.\n\n*   **Reversal/Continuation Signals:** The strong bullish move suggests a potential continuation of the upward trend, but the increased volatility indicates a higher risk of reversals.\n\nIn summary, the chart suggests a bullish trend, with a recent major price spike indicating significant buying pressure. However, increased volatility means potential for sharp reversals.\n"',
          },
        ],
        config: {},
      }),
    });

    const data = await res.json();
    console.log(data);
  });

  test("e2e", async () => {
    const memory = await makeMemory();
    // Add a single memory
    console.log("\nAdding a single memory...");
    const result1 = await memory.add(
      "Hi, my name is John and I am a software engineer.",
      {
        userId: "john",
      }
    );
    console.log("Added memory:", result1);

    // Add multiple messages
    console.log("\nAdding multiple messages...");
    const result2 = await memory.add(
      [
        { role: "user", content: "What is your favorite city?" },
        { role: "assistant", content: "I love Paris, it is my favorite city." },
      ],
      {
        userId: "john",
      }
    );
    console.log("Added messages:", result2);

    // Trying to update the memory
    const result3 = await memory.add(
      [
        { role: "user", content: "What is your favorite city?" },
        {
          role: "assistant",
          content: "I love New York, it is my favorite city.",
        },
      ],
      {
        userId: "john",
      }
    );
    console.log("Updated messages:", result3);

    // Get a single memory
    console.log("\nGetting a single memory...");
    if (result1.results && result1.results.length > 0) {
      const singleMemory = await memory.get(result1.results[0].id);
      console.log("Single memory:", singleMemory);
    } else {
      console.log("No memory was added in the first step");
    }

    // Get all memories before update
    console.log("\nGetting all memories before update...");
    const allMemoriesBefore = await memory.getAll({
      userId: "john",
    });
    console.log("All memories before update:", allMemoriesBefore);

    // Updating memory if we have any
    if (allMemoriesBefore.results && allMemoriesBefore.results.length > 0) {
      const result4 = await memory.update(
        allMemoriesBefore.results[0].id,
        "I love India, it is my favorite country."
      );
      console.log("Updated memory:", result4);
    } else {
      console.log("No memories found to update");
    }

    // Get all memories
    console.log("\nGetting all memories...");
    const allMemories = await memory.getAll({
      userId: "john",
    });
    console.log("All memories:", allMemories);

    // Search for memories
    console.log("\nSearching memories...");
    const searchResult = await memory.search("What do you know about Paris?", {
      userId: "john",
    });
    console.log("Search results:", searchResult);

    // Get memory history
    if (result1.results && result1.results.length > 0) {
      console.log("\nGetting memory history...");
      const history = await memory.history(result1.results[0].id);
      console.log("Memory history:", history);
    }

    // Delete a memory
    if (result1.results && result1.results.length > 0) {
      console.log("\nDeleting a memory...");
      await memory.delete(result1.results[0].id);
      console.log("Memory deleted successfully");
    }

    // Reset all memories
    console.log("\nResetting all memories...");
    await memory.reset();
    console.log("All memories reset");
  }, 100000);
});

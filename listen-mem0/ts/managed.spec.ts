import { makeMemoryManaged } from "./memory";

describe("managed", () => {
  test("e2e", async () => {
    const memory = await makeMemoryManaged();
    // Add a single memory
    console.log("\nAdding a single memory...");
    const result1 = await memory.add(
      "Hi, my name is John and I am a software engineer.",
      {
        user_id: "john",
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
        user_id: "john",
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
        user_id: "john",
      }
    );
    console.log("Updated messages:", result3);

    // Get a single memory
    console.log("\nGetting a single memory...");
    if (result1.length > 0) {
      const singleMemory = await memory.get(result1[0].id);
      console.log("Single memory:", singleMemory);
    } else {
      console.log("No memory was added in the first step");
    }

    // Get all memories before update
    console.log("\nGetting all memories before update...");
    const allMemoriesBefore = await memory.getAll({
      user_id: "john",
    });
    console.log("All memories before update:", allMemoriesBefore);

    // Updating memory if we have any
    if (allMemoriesBefore.length > 0) {
      const result4 = await memory.update(
        allMemoriesBefore[0].id,
        "I love India, it is my favorite country."
      );
      console.log("Updated memory:", result4);
    } else {
      console.log("No memories found to update");
    }

    // Get all memories
    console.log("\nGetting all memories...");
    const allMemories = await memory.getAll({
      user_id: "john",
    });
    console.log("All memories:", allMemories);

    // Search for memories
    console.log("\nSearching memories...");
    const searchResult = await memory.search("What do you know about Paris?", {
      user_id: "john",
    });
    console.log("Search results:", searchResult);

    // Get memory history
    if (result1.length > 0) {
      console.log("\nGetting memory history...");
      const history = await memory.history(result1[0].id);
      console.log("Memory history:", history);
    }

    // Delete a memory
    if (result1.length > 0) {
      console.log("\nDeleting a memory...");
      await memory.delete(result1[0].id);
      console.log("Memory deleted successfully");
    }
  }, 100000);
});

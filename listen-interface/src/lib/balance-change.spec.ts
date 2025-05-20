import { describe, expect, it } from "bun:test";
import { getBalanceChange } from "./balance-change";

describe("getBalanceChange", () => {
  it("should return the balance change for a given signature", async () => {
    const signature =
      "4KYhESYq1wr2eY2C3gqBCHNGosSSGu8EF2ijLaU8YDZLzmWUAESmMjJm34T8ZFyFxiaKXR6jbg1LVwbZthC43sYa";
    const userAddress = "6fp9frQ16W3kTRGiBVvpMS2NzoixE4Y1MWqYrW9SvTAj";
    const balanceChange = await getBalanceChange(signature, userAddress);
    expect(balanceChange).toEqual({
      symbol: "Fartcoin ",
      uiAmount: "1.331",
      pubkey: "9BB6NFEcjBCtnNLFko2FqVQBq8HHM13kCyYcdQbgpump",
    });
  }, 60000);
});

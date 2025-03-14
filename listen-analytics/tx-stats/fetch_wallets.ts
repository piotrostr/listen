import * as fs from "fs";

interface WalletResponse {
  wallets: any[];
  next_cursor?: string;
  [key: string]: any;
}

async function fetchAllWallets() {
  let cursor: string | null = null;
  let page = 1;
  let hasMoreData = true;

  while (hasMoreData) {
    // Construct URL with cursor if available
    const url: string = `https://dashboard.privy.io/api/v1/apps/cm6c7ifqd00ar52m1qxfgbkkn/wallets?limit=41${
      cursor ? `&cursor=${cursor}` : ""
    }`;

    console.log(`Fetching page ${page}, URL: ${url}`);

    const res: Response = await fetch(url, {
      headers: {
        accept: "application/json, text/plain, */*",
        "accept-language": "en-US,en;q=0.9",
        authorization: process.env.AUTH_TOKEN!,
        priority: "u=1, i",
        "privy-app-id": "cla04x0d00002nyb6oofp5dqh",
        "privy-client": "privy-dashboard:1.0.0",
        "sec-ch-ua":
          '"Not(A:Brand";v="99", "Google Chrome";v="133", "Chromium";v="133"',
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": '"macOS"',
        "sec-fetch-dest": "empty",
        "sec-fetch-mode": "cors",
        "sec-fetch-site": "same-origin",
      },
      referrer:
        "https://dashboard.privy.io/apps/cm6c7ifqd00ar52m1qxfgbkkn/wallets?wallet-tab=transactions",
      referrerPolicy: "strict-origin-when-cross-origin",
      body: null,
      method: "GET",
      mode: "cors",
      credentials: "include",
    });

    const data: WalletResponse = await res.json();

    // Save response to a JSON file
    fs.writeFileSync(
      `output/wallets_page_${page}.json`,
      JSON.stringify(data, null, 2),
    );
    console.log(`Saved page ${page} to output/wallets_page_${page}.json`);

    // Check if there's more data to fetch
    if (data.next_cursor) {
      cursor = data.next_cursor;
      page++;
    } else {
      hasMoreData = false;
      console.log("No more data to fetch");
    }

    // Optional: add a small delay to avoid rate limiting
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
}

// Run the function
fetchAllWallets()
  .then(() => console.log("Completed fetching all wallet data"))
  .catch((error) => console.error("Error fetching wallet data:", error));

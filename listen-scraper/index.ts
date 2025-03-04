import { researchX } from "./research";
import { TwitterApi } from "./twitterapi";

const main = async () => {
  const api = new TwitterApi(process.env.TWITTERAPI_API_KEY!);
  await researchX(
    api,
    "https://x.com/truth_terminal/status/1844470764583424360?s=46"
  );
};

await main();

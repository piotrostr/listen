import { TwitterApi } from "./twitterapi";

export const researchX = async (api: TwitterApi, url: string) => {
  try {
    const matchResult = matchUrl(url);
    console.log(`Processing ${matchResult.type} with ID: ${matchResult.id}`);

    if (matchResult.type === ResearchXUrlType.USER) {
      // Research a user
      const userInfo = await api.getUserInfo(matchResult.id);
      console.log("User Information:", userInfo);

      // Get their recent tweets
      const tweets = await api.getUserLastTweets({
        userName: matchResult.id,
        includeReplies: true,
      });

      console.log(
        `Retrieved ${tweets.data.tweets.length} tweets from user ${matchResult.id}`
      );
      return {
        type: "user",
        userInfo,
        tweets: tweets.data.tweets,
      };
    } else if (matchResult.type === ResearchXUrlType.POST) {
      // Research a specific tweet
      const tweetData = await api.getTweetsByIds([matchResult.id]);

      if (tweetData.length === 0) {
        throw new Error(`Tweet with ID ${matchResult.id} not found`);
      }

      const tweet = tweetData[0];
      console.log("Tweet found:", tweet);

      // Could add additional research on the tweet author here
      if (tweet.author) {
        console.log(
          `Tweet by ${tweet.author.name} (@${tweet.author.userName})`
        );
      }

      return {
        type: "tweet",
        tweet,
        // You could add more context data here if needed
      };
    }

    throw new Error(`Unsupported research type: ${matchResult.type}`);
  } catch (error) {
    console.error("Research failed:", error);
    throw error;
  }
};

enum ResearchXUrlType {
  POST = "post",
  USER = "user",
}

// this is likely going to require some more fuzzing
// also handle "wrong" urls, users might submit any url
const matchUrl = (url: string): { type: ResearchXUrlType; id: string } => {
  try {
    const urlObj = new URL(url);

    // Check if it's a Twitter/X domain
    if (!["twitter.com", "x.com"].includes(urlObj.hostname)) {
      throw new Error(
        `Invalid domain: ${urlObj.hostname}. Expected twitter.com or x.com`
      );
    }

    const pathParts = urlObj.pathname.split("/").filter(Boolean);

    // Case 1: Post URL format: twitter.com/username/status/tweetId
    if (pathParts.length >= 3 && pathParts[1] === "status") {
      return {
        type: ResearchXUrlType.POST,
        id: pathParts[2],
      };
    }

    // Case 2: User URL format: twitter.com/username
    if (pathParts.length === 1) {
      return {
        type: ResearchXUrlType.USER,
        id: pathParts[0],
      };
    }

    throw new Error(`Unrecognized Twitter/X URL format: ${url}`);
  } catch (error) {
    if (error instanceof Error) {
      throw new Error(`Failed to parse URL: ${error.message}`);
    }
    throw new Error("Failed to parse URL");
  }
};

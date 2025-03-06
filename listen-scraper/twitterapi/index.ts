import { z } from "zod";
import { TwitterApiClient } from "./client";
import { LastTweetsResponseSchema, TweetSchema, type Tweet } from "./tweet";
import { UserInfoResponseSchema, type UserInfo } from "./userInfo";

export class TwitterApi {
  private client: TwitterApiClient;

  constructor(apiKey: string) {
    this.client = new TwitterApiClient(apiKey);
  }

  /**
   * Get user information by username
   * @param userName The screen name of the user
   * @returns User information
   */
  async getUserInfo(userName: string): Promise<UserInfo> {
    const validatedResponse = await this.client.requestWithSchema(
      "/twitter/user/info",
      UserInfoResponseSchema,
      { userName }
    );
    return validatedResponse.data;
  }

  /**
   * Get user's last tweets
   * @param options Parameters for the request
   * @returns List of tweets and pagination info
   */
  async getUserLastTweets(options: {
    userId?: string;
    userName?: string;
    includeReplies?: boolean;
    cursor?: string;
  }) {
    if (!options.userId && !options.userName) {
      throw new Error("Either userId or userName must be provided");
    }

    const params: Record<string, string> = {};
    if (options.userId) params.userId = options.userId;
    if (options.userName) params.userName = options.userName;
    if (options.includeReplies !== undefined)
      params.includeReplies = options.includeReplies.toString();
    if (options.cursor) params.cursor = options.cursor;

    return await this.client.requestWithSchema(
      "/twitter/user/last_tweets",
      LastTweetsResponseSchema,
      params
    );
  }

  /**
   * Get tweets by their IDs
   * @param tweetIds Array of tweet IDs to fetch
   * @returns Array of tweet objects
   */
  async getTweetsByIds(tweetIds: string[]): Promise<Tweet[]> {
    if (!tweetIds.length) {
      throw new Error("At least one tweet ID must be provided");
    }

    const params: Record<string, string> = {
      tweet_ids: tweetIds.join(","),
    };

    const tweetsResponseSchema = z.object({
      tweets: z.array(TweetSchema),
      status: z.enum(["success", "error"]),
      msg: z.string(),
    });

    const validatedResponse = await this.client.requestWithSchema(
      "/twitter/tweets",
      tweetsResponseSchema,
      params
    );

    return validatedResponse.tweets;
  }
}

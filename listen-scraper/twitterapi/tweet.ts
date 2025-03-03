import { z } from "zod";
import { UserInfoSchema } from "./userInfo";
import { createApiResponseSchema } from "./util";

// Rename fields to match API response structure
export const TweetAuthorSchema = UserInfoSchema;

export const TweetEntityUrlSchema = z.object({
  display_url: z.string().optional(),
  expanded_url: z.string().optional(),
  url: z.string().optional(),
  indices: z.array(z.number()).optional(),
});

export const TweetEntitiesSchema = z.object({
  hashtags: z
    .array(
      z.object({
        text: z.string(),
        indices: z.array(z.number()),
      })
    )
    .optional(),
  urls: z.array(TweetEntityUrlSchema).optional(),
  user_mentions: z
    .array(
      z.object({
        id_str: z.string().optional(),
        name: z.string().optional(),
        screen_name: z.string().optional(),
      })
    )
    .optional(),
});

export const TweetSchema = z.object({
  type: z.string().optional(),
  id: z.string(),
  url: z.string().optional(),
  text: z.string(),
  source: z.string().optional(),
  retweetCount: z.number().optional(),
  replyCount: z.number().optional(),
  likeCount: z.number().optional(),
  quoteCount: z.number().optional(),
  viewCount: z.number().optional(),
  bookmarkCount: z.number().optional(),
  createdAt: z.string(),
  lang: z.string().optional(),
  isReply: z.boolean().optional(),
  inReplyToId: z.string().nullable().optional(),
  conversationId: z.string().optional(),
  inReplyToUserId: z.string().nullable().optional(),
  inReplyToUsername: z.string().nullable().optional(),
  author: TweetAuthorSchema.optional(),
  entities: TweetEntitiesSchema.optional(),
  quoted_tweet: z.any().nullable().optional(),
  retweeted_tweet: z.any().nullable().optional(),
});

// Define the data structure inside the API response
export const LastTweetsDataSchema = z.object({
  pin_tweet: z.any().nullable().optional(),
  tweets: z.array(TweetSchema),
});

// Create the full response schema using the utility function
export const LastTweetsResponseSchema =
  createApiResponseSchema(LastTweetsDataSchema);

// Type definitions based on schemas
export type Tweet = z.infer<typeof TweetSchema>;
export type LastTweetsData = z.infer<typeof LastTweetsDataSchema>;
export type LastTweetsResponse = z.infer<typeof LastTweetsResponseSchema>;

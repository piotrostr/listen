import { z } from "zod";

// Simple entities
export const HashtagSchema = z.object({
  indices: z.array(z.number()),
  text: z.string(),
});

export const UserMentionSchema = z.object({
  id_str: z.string(),
  name: z.string(),
  screen_name: z.string(),
});

export const UrlEntitySchema = z.object({
  display_url: z.string(),
  expanded_url: z.string(),
  indices: z.array(z.number()),
  url: z.string(),
});

export const EntityDescriptionSchema = z.object({
  urls: z.array(UrlEntitySchema).optional().nullable(),
});

export const ProfileBioEntitiesSchema = z.object({
  description: EntityDescriptionSchema.optional().nullable(),
  url: EntityDescriptionSchema.optional().nullable(),
});

export const ProfileBioSchema = z.object({
  description: z.string().optional().nullable(),
  entities: ProfileBioEntitiesSchema.optional().nullable(),
});

// User info
export const UserInfoSchema = z.object({
  type: z.string().optional().default("user"),
  userName: z.string(),
  url: z.string().optional().nullable(),
  id: z.string(),
  name: z.string(),
  isVerified: z.boolean().optional().nullable(),
  isBlueVerified: z.boolean().optional().nullable(),
  profilePicture: z.string().optional().nullable(),
  coverPicture: z.string().optional().nullable(),
  description: z.string().optional().nullable(),
  location: z.string().optional().nullable(),
  followers: z.number().optional().nullable(),
  following: z.number().optional().nullable(),
  canDm: z.boolean().optional().nullable(),
  createdAt: z.string().optional().nullable(),
  fastFollowersCount: z.number().optional().nullable(),
  favouritesCount: z.number().optional().nullable(),
  hasCustomTimelines: z.boolean().optional().nullable(),
  isTranslator: z.boolean().optional().nullable(),
  mediaCount: z.number().optional().nullable(),
  statusesCount: z.number().optional().nullable(),
  withheldInCountries: z.array(z.string()).optional().nullable(),
  affiliatesHighlightedLabel: z.record(z.unknown()).optional().nullable(),
  possiblySensitive: z.boolean().optional().nullable(),
  pinnedTweetIds: z.array(z.string()).optional().nullable(),
  isAutomated: z.boolean().optional().nullable(),
  automatedBy: z.string().nullable().optional(),
  unavailable: z.boolean().nullable().optional(),
  message: z.string().nullable().optional(),
  unavailableReason: z.string().nullable().optional(),
  profileBio: ProfileBioSchema.nullable().optional(),
});

// Tweet entities
export const TweetEntitiesSchema = z.object({
  hashtags: z.array(HashtagSchema).optional().nullable(),
  urls: z.array(UrlEntitySchema).optional().nullable(),
  user_mentions: z.array(UserMentionSchema).optional().nullable(),
});

// Self-referential Tweet schema
export const TweetSchema: z.ZodType<any> = z.lazy(() =>
  z.object({
    type: z.string().optional().default("tweet"),
    id: z.string(),
    url: z.string().optional().nullable(),
    text: z.string(),
    source: z.string().optional().nullable(),
    retweetCount: z.number().optional().nullable(),
    replyCount: z.number().optional().nullable(),
    likeCount: z.number().optional().nullable(),
    quoteCount: z.number().optional().nullable(),
    viewCount: z.number().optional().nullable(),
    createdAt: z.string(),
    lang: z.string().optional().nullable(),
    bookmarkCount: z.number().optional().nullable(),
    isReply: z.boolean().optional().nullable(),
    inReplyToId: z.string().nullable().optional(),
    conversationId: z.string().optional().nullable(),
    inReplyToUserId: z.string().optional().nullable(),
    inReplyToUsername: z.string().optional().nullable(),
    author: UserInfoSchema.optional().nullable(),
    entities: TweetEntitiesSchema.optional().nullable(),
    quotedTweet: TweetSchema.nullable().optional().nullable(),
    retweetedTweet: TweetSchema.nullable().optional().nullable(),
  })
);

// Type exports
export type Hashtag = z.infer<typeof HashtagSchema>;
export type UserMention = z.infer<typeof UserMentionSchema>;
export type UrlEntity = z.infer<typeof UrlEntitySchema>;
export type EntityDescription = z.infer<typeof EntityDescriptionSchema>;
export type ProfileBioEntities = z.infer<typeof ProfileBioEntitiesSchema>;
export type ProfileBio = z.infer<typeof ProfileBioSchema>;
export type UserInfo = z.infer<typeof UserInfoSchema>;
export type TweetEntities = z.infer<typeof TweetEntitiesSchema>;
export type Tweet = z.infer<typeof TweetSchema>;

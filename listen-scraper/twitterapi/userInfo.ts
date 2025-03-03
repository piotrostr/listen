import { z } from "zod";
import { createApiResponseSchema } from "./util";

export const ProfileBioUrlSchema = z.object({
  display_url: z.string(),
  expanded_url: z.string(),
  indices: z.array(z.number()),
  url: z.string(),
});

export const ProfileBioEntitiesSchema = z.object({
  description: z
    .object({
      urls: z.array(ProfileBioUrlSchema).optional(),
    })
    .optional(),
  url: z
    .object({
      urls: z.array(ProfileBioUrlSchema).optional(),
    })
    .optional(),
});

export const ProfileBioSchema = z.object({
  description: z.string().optional(),
  entities: ProfileBioEntitiesSchema.optional(),
});

export const UserInfoSchema = z.object({
  type: z.string().optional(),
  userName: z.string(),
  url: z.string().optional(),
  id: z.string(),
  name: z.string(),
  isBlueVerified: z.boolean().optional(),
  profilePicture: z.string().optional(),
  coverPicture: z.string().optional(),
  description: z.string().optional(),
  location: z.string().optional(),
  followers: z.number().optional(),
  following: z.number().optional(),
  canDm: z.boolean().optional(),
  createdAt: z.string().optional(),
  fastFollowersCount: z.number().optional(),
  favouritesCount: z.number().optional(),
  hasCustomTimelines: z.boolean().optional(),
  isTranslator: z.boolean().optional(),
  mediaCount: z.number().optional(),
  statusesCount: z.number().optional(),
  withheldInCountries: z.array(z.string()).optional(),
  affiliatesHighlightedLabel: z.record(z.any()).optional(),
  possiblySensitive: z.boolean().optional(),
  pinnedTweetIds: z.array(z.string()).optional(),
  isAutomated: z.boolean().optional(),
  automatedBy: z.string().nullable().optional(),
  unavailable: z.boolean().optional(),
  message: z.string().optional(),
  unavailableReason: z.string().optional(),
  profile_bio: ProfileBioSchema.optional(),
});

// Type definitions based on schemas
export type UserInfo = z.infer<typeof UserInfoSchema>;

// Create the specific schema for UserInfo responses
export const UserInfoResponseSchema = createApiResponseSchema(UserInfoSchema);

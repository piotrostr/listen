import { z } from "zod";

// Create a generic API response schema
export const createApiResponseSchema = <T extends z.ZodTypeAny>(schema: T) =>
  z.object({
    data: schema,
    status: z.enum(["success", "error"]),
    msg: z.string(),
  });

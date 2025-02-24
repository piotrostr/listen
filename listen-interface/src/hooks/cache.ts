import { z } from "zod";

export interface CacheStore<T> {
  schema: z.ZodType<T>;
  get(key: string): Promise<T | null>;
  set(key: string, value: T): Promise<void>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
  getAll(): Promise<T[]>;
}

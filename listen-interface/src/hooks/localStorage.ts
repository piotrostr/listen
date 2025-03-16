import { z } from "zod";
import { Chat, ChatSchema } from "../types/message";
import { CacheStore } from "./cache";
import { TokenMetadata, TokenMetadataSchema } from "./types";

export class LocalStorageCache<T> implements CacheStore<T> {
  private storeName: string;
  private maxSize?: number;
  private ttl?: number; // milliseconds
  public schema: z.ZodType<any, any, any>;

  constructor(
    storeName: string,
    schema: z.ZodType<any, any, any>,
    options: { maxSize?: number; ttl?: number } = {}
  ) {
    this.storeName = storeName;
    this.schema = schema;
    this.maxSize = options.maxSize;
    this.ttl = options.ttl;
  }

  private getFullKey(key: string): string {
    return `${this.storeName}:${key}`;
  }

  private deserializeDates(obj: any): any {
    if (obj === null || obj === undefined) return obj;
    if (typeof obj !== "object") return obj;

    for (const key in obj) {
      // Skip the message field since it contains strings that should not be parsed
      if (key === "message") continue;

      const value = obj[key];
      if (typeof value === "string") {
        // Try to parse ISO date strings
        const date = new Date(value);
        if (!isNaN(date.getTime()) && value.includes("T")) {
          obj[key] = date;
        }
      } else if (typeof value === "object") {
        obj[key] = this.deserializeDates(value);
      }
    }
    return obj;
  }

  async get(key: string): Promise<T | null> {
    const fullKey = this.getFullKey(key);
    const item = localStorage.getItem(fullKey);

    if (!item) return null;

    const parsed = this.deserializeDates(JSON.parse(item));

    if (this.ttl) {
      if (parsed.expires && parsed.expires < Date.now()) {
        this.delete(key);
        return null;
      }
      return this.schema.parse(parsed.value);
    }

    return this.schema.parse(parsed);
  }

  async set(key: string, value: T): Promise<void> {
    const fullKey = this.getFullKey(key);
    // Validate value before storing
    const validated = this.schema.parse(value);

    if (this.ttl) {
      const entry = {
        value: validated,
        timestamp: Date.now(),
        expires: Date.now() + this.ttl,
      };
      localStorage.setItem(fullKey, JSON.stringify(entry));
    } else {
      localStorage.setItem(fullKey, JSON.stringify(validated));
    }

    if (this.maxSize) {
      const allKeys = await this.getAllKeys();
      if (allKeys.length > this.maxSize) {
        // Remove oldest entries
        const keysToRemove = allKeys.slice(0, allKeys.length - this.maxSize);
        keysToRemove.forEach((k) => this.delete(k));
      }
    }
  }

  async delete(key: string): Promise<void> {
    const fullKey = this.getFullKey(key);
    localStorage.removeItem(fullKey);
  }

  async clear(): Promise<void> {
    const allKeys = await this.getAllKeys();
    allKeys.forEach((key) => this.delete(key));
  }

  private async getAllKeys(): Promise<string[]> {
    return Object.keys(localStorage)
      .filter((key) => key.startsWith(`${this.storeName}:`))
      .map((key) => key.replace(`${this.storeName}:`, ""));
  }

  async getAll(): Promise<T[]> {
    const allKeys = await this.getAllKeys();
    const values: T[] = [];

    for (const key of allKeys) {
      const value = await this.get(key);
      if (value !== null) {
        values.push(value);
      }
    }

    return values;
  }
}

// Create instances for different types of data
export const tokenMetadataCache = new LocalStorageCache<TokenMetadata>(
  "token-metadata",
  TokenMetadataSchema,
  { ttl: 24 * 60 * 60 * 1000 } // 24 hours TTL
);

// Add cache instance for chats
export const chatCache = new LocalStorageCache<Chat>(
  "chats",
  ChatSchema,
  { maxSize: 1000 } // Store up to 1000 chats
);

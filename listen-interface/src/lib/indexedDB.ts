import { openDB, type IDBPDatabase } from "idb";
import { z } from "zod";
import { Chat, ChatSchema } from "../types/message";
import { CacheStore } from "./cache";
import { TokenMetadata, TokenMetadataSchema } from "./types";

export class IndexedDBCache<T> implements CacheStore<T> {
  private dbName: string;
  private storeName: string;
  private db: Promise<IDBPDatabase>;
  private maxSize?: number;
  private ttl?: number; // milliseconds
  public schema: z.ZodType<any, any, any>;

  constructor(
    dbName: string,
    storeName: string,
    schema: z.ZodType<any, any, any>,
    options: { maxSize?: number; ttl?: number } = {}
  ) {
    this.dbName = dbName;
    this.storeName = storeName;
    this.schema = schema;
    this.maxSize = options.maxSize;
    this.ttl = options.ttl;

    // Initialize the database connection
    this.db = openDB(this.dbName, 1, {
      upgrade(db) {
        if (!db.objectStoreNames.contains(storeName)) {
          db.createObjectStore(storeName);
        }
      },
    });
  }

  private deserializeDates(obj: any): any {
    if (obj === null || obj === undefined) return obj;
    if (typeof obj !== "object") return obj;

    for (const key in obj) {
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
    const db = await this.db;
    const result = await db.get(this.storeName, key);

    if (!result) return null;

    const deserialized = this.deserializeDates(result);

    if (this.ttl) {
      const entry = deserialized as {
        value: T;
        timestamp: number;
        expires: number;
      };
      if (entry.expires && entry.expires < Date.now()) {
        this.delete(key);
        return null;
      }
      return this.schema.parse(entry.value);
    }

    return this.schema.parse(deserialized);
  }

  async set(key: string, value: T): Promise<void> {
    const db = await this.db;
    // Validate value before storing
    const validated = this.schema.parse(value);

    if (this.ttl) {
      const entry = {
        value: validated,
        timestamp: Date.now(),
        expires: Date.now() + this.ttl,
      };
      await db.put(this.storeName, entry, key);
    } else {
      await db.put(this.storeName, validated, key);
    }

    if (this.maxSize) {
      const keys = await db.getAllKeys(this.storeName);
      if (keys.length > this.maxSize) {
        // Remove oldest entries
        const keysToRemove = keys.slice(0, keys.length - this.maxSize);
        await Promise.all(
          keysToRemove.map((k) => db.delete(this.storeName, k))
        );
      }
    }
  }

  async delete(key: string): Promise<void> {
    const db = await this.db;
    await db.delete(this.storeName, key);
  }

  async clear(): Promise<void> {
    const db = await this.db;
    await db.clear(this.storeName);
  }

  async getAll(): Promise<T[]> {
    const db = await this.db;
    const results = await db.getAll(this.storeName);

    return results.map((result) => {
      const deserialized = this.deserializeDates(result);
      if (this.ttl) {
        const entry = deserialized as {
          value: T;
          timestamp: number;
          expires: number;
        };
        return this.schema.parse(entry.value);
      }
      return this.schema.parse(deserialized);
    });
  }

  async close(): Promise<void> {
    const db = await this.db;
    db.close();
  }
}

// Create instances for different types of data
export const tokenMetadataCache = new IndexedDBCache<TokenMetadata>(
  "listen-db",
  "token-metadata",
  TokenMetadataSchema,
  { ttl: 24 * 60 * 60 * 1000 } // 24 hours TTL
);

// Add new cache instance for chats
export const chatCache = new IndexedDBCache<Chat>(
  "listen-db",
  "chats",
  ChatSchema,
  { maxSize: 1000 } // Store up to 1000 chats
);

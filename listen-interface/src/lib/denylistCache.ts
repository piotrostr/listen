// Persistent cache for denylisted tokens using IndexedDB with TTL
interface DenylistEntry {
  key: string;
  timestamp: number;
}

class DenylistCache {
  private readonly DB_NAME = 'TokenDenylistDB';
  private readonly DB_VERSION = 1;
  private readonly STORE_NAME = 'denylisted_tokens';
  private readonly TTL_MS = 24 * 60 * 60 * 1000; // 24 hours
  private readonly DENYLIST_MESSAGES = ["deny list", "denylist", "is invalid"];
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;

  /**
   * Initialize IndexedDB
   */
  private async initDB(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(this.DB_NAME, this.DB_VERSION);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        if (!db.objectStoreNames.contains(this.STORE_NAME)) {
          const store = db.createObjectStore(this.STORE_NAME, { keyPath: 'key' });
          store.createIndex('timestamp', 'timestamp', { unique: false });
        }
      };
    });

    return this.initPromise;
  }

  /**
   * Create a standardized key for a token
   */
  private createKey(token: string, chainId: string | number): string {
    return `${chainId}-${token.toLowerCase()}`;
  }

  /**
   * Check if an entry is expired
   */
  private isExpired(timestamp: number): boolean {
    return Date.now() - timestamp > this.TTL_MS;
  }

  /**
   * Clean up expired entries
   */
  private async cleanupExpired(): Promise<void> {
    try {
      await this.initDB();
      if (!this.db) return;

      const transaction = this.db.transaction([this.STORE_NAME], 'readwrite');
      const store = transaction.objectStore(this.STORE_NAME);
      
      const index = store.index('timestamp');
      const cutoffTime = Date.now() - this.TTL_MS;
      const range = IDBKeyRange.upperBound(cutoffTime);
      
      const request = index.openCursor(range);
      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;
        if (cursor) {
          cursor.delete();
          cursor.continue();
        }
      };
    } catch (error) {
      console.warn('Failed to cleanup expired denylist entries:', error);
    }
  }

  /**
   * Check if a token is in the denylist
   */
  async isDenylisted(token: string, chainId: string | number): Promise<boolean> {
    try {
      await this.initDB();
      if (!this.db) return false;

      const key = this.createKey(token, chainId);
      const transaction = this.db.transaction([this.STORE_NAME], 'readonly');
      const store = transaction.objectStore(this.STORE_NAME);
      
      return new Promise((resolve) => {
        const request = store.get(key);
        request.onsuccess = () => {
          const result = request.result as DenylistEntry | undefined;
          if (!result) {
            resolve(false);
            return;
          }

          // Check if entry is expired
          if (this.isExpired(result.timestamp)) {
            // Remove expired entry
            this.removeFromDenylist(token, chainId);
            resolve(false);
            return;
          }

          resolve(true);
        };
        request.onerror = () => resolve(false);
      });
    } catch (error) {
      console.warn('Failed to check denylist:', error);
      return false;
    }
  }

  /**
   * Add a token to the denylist
   */
  async addToDenylist(token: string, chainId: string | number): Promise<void> {
    try {
      await this.initDB();
      if (!this.db) return;

      const key = this.createKey(token, chainId);
      const entry: DenylistEntry = {
        key,
        timestamp: Date.now()
      };

      const transaction = this.db.transaction([this.STORE_NAME], 'readwrite');
      const store = transaction.objectStore(this.STORE_NAME);
      
      await new Promise<void>((resolve, reject) => {
        const request = store.put(entry);
        request.onsuccess = () => {
          console.log(`Added to denylist: ${key} (TTL: 24h)`);
          resolve();
        };
        request.onerror = () => reject(request.error);
      });

      // Periodically cleanup expired entries
      if (Math.random() < 0.1) { // 10% chance
        this.cleanupExpired();
      }
    } catch (error) {
      console.warn(`Failed to add ${token} to denylist:`, error);
    }
  }

  /**
   * Remove a token from the denylist
   */
  private async removeFromDenylist(token: string, chainId: string | number): Promise<void> {
    try {
      if (!this.db) return;

      const key = this.createKey(token, chainId);
      const transaction = this.db.transaction([this.STORE_NAME], 'readwrite');
      const store = transaction.objectStore(this.STORE_NAME);
      store.delete(key);
    } catch (error) {
      console.warn(`Failed to remove ${token} from denylist:`, error);
    }
  }

  /**
   * Check if an error message indicates a denylisted token
   */
  isDenylistError(errorMessage: string): boolean {
    const message = errorMessage.toLowerCase();
    return this.DENYLIST_MESSAGES.some((phrase) => message.includes(phrase));
  }

  /**
   * Parse a LiFi error message to extract token and chain info
   * Example: "Token 42161-0x42c35b14e6f163ea0a625c919dd95f5a3a594c19 is invalid or in deny list."
   */
  parseTokenFromError(
    errorMessage: string,
  ): { token: string; chainId: string } | null {
    const tokenMatch = errorMessage.match(/token\s+(\d+)-([a-fA-F0-9x]+)/i);
    if (tokenMatch) {
      return {
        chainId: tokenMatch[1],
        token: tokenMatch[2],
      };
    }
    return null;
  }

  /**
   * Handle a potential denylist error and cache if needed
   */
  async handlePotentialDenylistError(
    errorMessage: string,
    token?: string,
    chainId?: string | number,
  ): Promise<boolean> {
    if (!this.isDenylistError(errorMessage)) {
      return false;
    }

    // Try to parse token info from error message
    const parsed = this.parseTokenFromError(errorMessage);
    if (parsed) {
      await this.addToDenylist(parsed.token, parsed.chainId);
      return true;
    }

    // If we have explicit token/chainId, use those
    if (token && chainId) {
      await this.addToDenylist(token, chainId);
      return true;
    }

    return false;
  }

  /**
   * Get current denylist size (for debugging)
   */
  async size(): Promise<number> {
    try {
      await this.initDB();
      if (!this.db) return 0;

      const transaction = this.db.transaction([this.STORE_NAME], 'readonly');
      const store = transaction.objectStore(this.STORE_NAME);
      
      return new Promise((resolve) => {
        const request = store.count();
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => resolve(0);
      });
    } catch (error) {
      console.warn('Failed to get denylist size:', error);
      return 0;
    }
  }

  /**
   * Clear the denylist (for debugging/testing)
   */
  async clear(): Promise<void> {
    try {
      await this.initDB();
      if (!this.db) return;

      const transaction = this.db.transaction([this.STORE_NAME], 'readwrite');
      const store = transaction.objectStore(this.STORE_NAME);
      
      await new Promise<void>((resolve, reject) => {
        const request = store.clear();
        request.onsuccess = () => {
          console.log('Denylist cleared');
          resolve();
        };
        request.onerror = () => reject(request.error);
      });
    } catch (error) {
      console.warn('Failed to clear denylist:', error);
    }
  }
}

// Export singleton instance
export const tokenDenylistCache = new DenylistCache();

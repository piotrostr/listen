interface RateLimiterOptions {
  maxRequests: number;
  windowMs: number;
  retryAfterMs?: number;
  maxRetries?: number;
}

export class RateLimiter {
  private requests: number[] = [];
  private options: Required<RateLimiterOptions>;

  constructor(options: RateLimiterOptions) {
    this.options = {
      maxRequests: options.maxRequests,
      windowMs: options.windowMs,
      retryAfterMs: options.retryAfterMs || 1000,
      maxRetries: options.maxRetries || 3,
    };
  }

  private cleanupOldRequests() {
    const now = Date.now();
    this.requests = this.requests.filter(
      (timestamp) => now - timestamp < this.options.windowMs
    );
  }

  private async wait(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    let retries = 0;

    while (retries <= this.options.maxRetries) {
      this.cleanupOldRequests();

      if (this.requests.length < this.options.maxRequests) {
        this.requests.push(Date.now());
        
        try {
          const result = await fn();
          return result;
        } catch (error: any) {
          if (error.message?.includes('429') || error.status === 429) {
            retries++;
            if (retries > this.options.maxRetries) {
              throw error;
            }
            
            const backoffTime = this.options.retryAfterMs * Math.pow(2, retries - 1);
            console.warn(`Rate limited, retrying in ${backoffTime}ms (attempt ${retries}/${this.options.maxRetries})`);
            await this.wait(backoffTime);
            continue;
          }
          throw error;
        }
      } else {
        const oldestRequest = Math.min(...this.requests);
        const waitTime = this.options.windowMs - (Date.now() - oldestRequest) + 100;
        
        if (waitTime > 0) {
          await this.wait(waitTime);
        }
      }
    }

    throw new Error(`Failed after ${this.options.maxRetries} retries`);
  }
}

export const jupiterRateLimiter = new RateLimiter({
  maxRequests: 10,
  windowMs: 1000,
  retryAfterMs: 2000,
  maxRetries: 3,
});
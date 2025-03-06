import { z } from "zod";

export type ApiResponseSuccess<T> = {
  data: T;
  status: "success" | "error";
  msg: string;
};

export type ApiResponseError = {
  error: number;
  message: string;
};

export type ApiResponse<T> = ApiResponseSuccess<T> | ApiResponseError;

// Twitter API Client
export class TwitterApiClient {
  private apiKey: string;
  private baseUrl: string;

  constructor(apiKey: string, baseUrl = "https://api.twitterapi.io") {
    this.apiKey = apiKey;
    this.baseUrl = baseUrl;
  }

  async request<T>(
    endpoint: string,
    params?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    const url = new URL(`${this.baseUrl}${endpoint}`);

    // Add query parameters if provided
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value) url.searchParams.append(key, value);
      });
    }

    try {
      const response = await fetch(url.toString(), {
        method: "GET",
        headers: {
          "X-API-Key": this.apiKey,
        },
      });

      if (!response.ok) {
        const res = {
          error: response.status,
          message: response.statusText,
        };

        return res as ApiResponseError;
      }

      const data = await response.json();

      // Check if it's an error response
      if ("error" in data) {
        return data as ApiResponseError;
      }

      // Otherwise return as success
      return data as ApiResponseSuccess<T>;
    } catch (error) {
      // Handle unexpected errors
      return {
        error: 500,
        message: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Makes a request and validates the response with a Zod schema
   * @param endpoint API endpoint
   * @param schema Zod schema to validate the response
   * @param params Request parameters
   * @returns Validated data of type inferred from the schema
   */
  async requestWithSchema<T extends z.ZodType>(
    endpoint: string,
    schema: T,
    params?: Record<string, string>
  ): Promise<z.infer<T>> {
    const response = await this.request(endpoint, params);

    // Check for error response
    if ("error" in response) {
      throw new Error(`API Error (${response.error}): ${response.message}`);
    }

    // Validate with the provided schema
    try {
      return schema.parse(response);
    } catch (error) {
      if (error instanceof z.ZodError) {
        const errorMessage = error.errors
          .map((err) => `${err.path.join(".")}: ${err.message}`)
          .join(", ");
        throw new Error(`Response validation error: ${errorMessage}`);
      }
      throw new Error(
        `Response validation error: ${
          error instanceof Error ? error.message : "Unknown validation error"
        }`
      );
    }
  }
}

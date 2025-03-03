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
  ): Promise<T> {
    const url = new URL(`${this.baseUrl}${endpoint}`);

    // Add query parameters if provided
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value) url.searchParams.append(key, value);
      });
    }

    const response = await fetch(url.toString(), {
      method: "GET",
      headers: {
        "X-API-Key": this.apiKey,
      },
    });

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();

    // Check for API error response
    if (data.status === "error") {
      throw new Error(`API Error: ${data.msg}`);
    }

    return data as T;
  }
}

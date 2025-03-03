import { TwitterApiClient, type ApiResponse } from "./client";
import { UserInfoResponseSchema, type UserInfo } from "./userInfo";

export class TwitterApi {
  private client: TwitterApiClient;

  constructor(apiKey: string) {
    this.client = new TwitterApiClient(apiKey);
  }

  /**
   * Get user information by username
   * @param userName The screen name of the user
   * @returns User information
   */
  async getUserInfo(userName: string): Promise<UserInfo> {
    const response = await this.client.request<ApiResponse<UserInfo>>(
      "/twitter/user/info",
      {
        userName,
      }
    );

    // Validate the response with Zod
    const validatedResponse = UserInfoResponseSchema.parse(response);
    return validatedResponse.data;
  }
}

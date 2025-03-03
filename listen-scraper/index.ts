import { TwitterApi } from "./twitterapi";

const main = async () => {
  const api = new TwitterApi(process.env.TWITTERAPI_API_KEY!);
  const userInfo = await api.getUserInfo("listenonsol");
  console.log(userInfo);
  const tweets = await api.getUserLastTweets({
    userName: "listenonsol",
    includeReplies: true,
  });
  console.log(tweets.data.tweets);
};

main();

import { useEffect, useRef } from "react";
import i18n from "../i18n";
import { Tweet } from "../types/x";

export function FetchXPostDisplay({ tweet }: { tweet: Tweet }) {
  const tweetRef = useRef<HTMLDivElement>(null);
  console.debug(tweet);

  useEffect(() => {
    // Initialize tweet if Twitter widgets API is loaded
    if ((window as any).twttr && tweetRef.current) {
      (window as any).twttr.widgets.load(tweetRef.current);
    }
  }, [tweet.id]);

  return (
    <div
      ref={tweetRef}
      className="w-full h-full flex justify-center items-center bg-transparent"
    >
      <blockquote
        className="twitter-tweet"
        data-lang={i18n.language === "zh" ? "zh-cn" : "en"}
        data-theme="light"
      >
        <a
          href={`https://twitter.com/${tweet.author?.userName}/status/${tweet.id}`}
        ></a>
      </blockquote>
    </div>
  );
}

export function _FetchXPostDisplay({ tweet }: { tweet: Tweet }) {
  console.debug(tweet);
  return (
    <div
      className="border rounded-lg p-4 max-w-3xl hover:shadow-md transition-shadow duration-200 bg-black"
      style={{ borderColor: "rgb(47, 51, 54)" }}
    >
      {/* Author section */}
      <div className="flex items-start mb-3">
        <a
          href={tweet.author?.url || `https://x.com/${tweet.author?.userName}`}
          target="_blank"
          rel="noopener noreferrer"
          className="hover:opacity-90 transition-opacity"
        >
          <img
            src={
              tweet.author?.profilePicture ||
              "https://abs.twimg.com/sticky/default_profile_images/default_profile_normal.png"
            }
            alt={tweet.author?.name || "Profile"}
            className="w-12 h-12 rounded-full mr-3 cursor-pointer"
          />
        </a>
        <div>
          <div className="flex items-center">
            <a
              href={
                tweet.author?.url || `https://x.com/${tweet.author?.userName}`
              }
              target="_blank"
              rel="noopener noreferrer"
              className="font-bold hover:underline cursor-pointer"
            >
              {tweet.author?.name}
            </a>
            {tweet.author?.isBlueVerified && (
              <svg
                className="w-4 h-4 ml-1 text-blue-500"
                fill="currentColor"
                viewBox="0 0 24 24"
              >
                <path d="M22.5 12.5c0-1.58-.875-2.95-2.148-3.6.154-.435.238-.905.238-1.4 0-2.21-1.71-3.998-3.818-3.998-.47 0-.92.084-1.336.25C14.818 2.415 13.51 1.5 12 1.5s-2.816.917-3.437 2.25c-.415-.165-.866-.25-1.336-.25-2.11 0-3.818 1.79-3.818 4 0 .494.083.964.237 1.4-1.272.65-2.147 2.018-2.147 3.6 0 1.495.782 2.798 1.942 3.486-.02.17-.032.34-.032.514 0 2.21 1.708 4 3.818 4 .47 0 .92-.086 1.335-.25.62 1.334 1.926 2.25 3.437 2.25 1.512 0 2.818-.916 3.437-2.25.415.163.865.248 1.336.248 2.11 0 3.818-1.79 3.818-4 0-.174-.012-.344-.033-.513 1.158-.687 1.943-1.99 1.943-3.484zm-6.616-3.334l-4.334 6.5c-.145.217-.382.334-.625.334-.143 0-.288-.04-.416-.126l-.115-.094-2.415-2.415c-.293-.293-.293-.768 0-1.06s.768-.294 1.06 0l1.77 1.767 3.825-5.74c.23-.345.696-.436 1.04-.207.346.23.44.696.21 1.04z" />
              </svg>
            )}
            {tweet.author?.isVerified && !tweet.author?.isBlueVerified && (
              <svg
                className="w-4 h-4 ml-1 text-blue-500"
                fill="currentColor"
                viewBox="0 0 24 24"
              >
                <path d="M22.5 12.5c0-1.58-.875-2.95-2.148-3.6.154-.435.238-.905.238-1.4 0-2.21-1.71-3.998-3.818-3.998-.47 0-.92.084-1.336.25C14.818 2.415 13.51 1.5 12 1.5s-2.816.917-3.437 2.25c-.415-.165-.866-.25-1.336-.25-2.11 0-3.818 1.79-3.818 4 0 .494.083.964.237 1.4-1.272.65-2.147 2.018-2.147 3.6 0 1.495.782 2.798 1.942 3.486-.02.17-.032.34-.032.514 0 2.21 1.708 4 3.818 4 .47 0 .92-.086 1.335-.25.62 1.334 1.926 2.25 3.437 2.25 1.512 0 2.818-.916 3.437-2.25.415.163.865.248 1.336.248 2.11 0 3.818-1.79 3.818-4 0-.174-.012-.344-.033-.513 1.158-.687 1.943-1.99 1.943-3.484zm-6.616-3.334l-4.334 6.5c-.145.217-.382.334-.625.334-.143 0-.288-.04-.416-.126l-.115-.094-2.415-2.415c-.293-.293-.293-.768 0-1.06s.768-.294 1.06 0l1.77 1.767 3.825-5.74c.23-.345.696-.436 1.04-.207.346.23.44.696.21 1.04z" />
              </svg>
            )}
          </div>
          <a
            href={
              tweet.author?.url || `https://x.com/${tweet.author?.userName}`
            }
            target="_blank"
            rel="noopener noreferrer"
            className="text-gray-500 cursor-pointer"
          >
            @{tweet.author?.userName}
          </a>
        </div>
      </div>

      {/* Tweet content */}
      <div className="mb-4">
        <p className="text-lg">{tweet.text}</p>
      </div>

      {/* Quoted or retweeted content */}
      {tweet.quotedTweet && (
        <div
          className="border rounded-lg p-3 mb-3 hover:bg-gray-50 transition-colors duration-200"
          style={{ borderColor: "rgb(47, 51, 54)" }}
        >
          <div className="flex items-center mb-2">
            <a
              href={
                tweet.quotedTweet.author?.url ||
                `https://x.com/${tweet.quotedTweet.author?.userName}`
              }
              target="_blank"
              rel="noopener noreferrer"
              className="hover:opacity-90 transition-opacity"
            >
              <img
                src={
                  tweet.quotedTweet.author?.profilePicture ||
                  "https://abs.twimg.com/sticky/default_profile_images/default_profile_normal.png"
                }
                alt={tweet.quotedTweet.author?.name || "Profile"}
                className="w-8 h-8 rounded-full mr-2 cursor-pointer"
              />
            </a>
            <div>
              <a
                href={
                  tweet.quotedTweet.author?.url ||
                  `https://x.com/${tweet.quotedTweet.author?.userName}`
                }
                target="_blank"
                rel="noopener noreferrer"
                className="font-bold text-sm hover:underline cursor-pointer"
              >
                {tweet.quotedTweet.author?.name}
              </a>
              <a
                href={
                  tweet.quotedTweet.author?.url ||
                  `https://x.com/${tweet.quotedTweet.author?.userName}`
                }
                target="_blank"
                rel="noopener noreferrer"
                className="text-gray-500 text-sm ml-1 hover:underline cursor-pointer"
              >
                @{tweet.quotedTweet.author?.userName}
              </a>
            </div>
          </div>
          <p className="text-sm">{tweet.quotedTweet.text}</p>
        </div>
      )}

      {/* Date and metrics */}
      <a
        href={
          tweet.url ||
          `https://x.com/${tweet.author?.userName}/status/${tweet.id}`
        }
        target="_blank"
        rel="noopener noreferrer"
        className="block text-gray-500 text-sm mb-3 hover:underline cursor-pointer"
      >
        {tweet.createdAt && formatTwitterDate(new Date(tweet.createdAt))}
        {tweet.viewCount && (
          <>
            &nbsp;·&nbsp;
            <strong>
              {tweet.viewCount >= 1000000
                ? (tweet.viewCount / 1000000).toFixed(1).replace(/\.0$/, "") +
                  "M"
                : tweet.viewCount >= 1000
                  ? (tweet.viewCount / 1000).toFixed(1).replace(/\.0$/, "") +
                    "K"
                  : tweet.viewCount.toString()}{" "}
              Views
            </strong>
          </>
        )}
      </a>

      <div
        className="flex justify-between text-gray-500 border-t border-b py-2 text-sm"
        style={{ borderColor: "rgb(47, 51, 54)" }}
      >
        <a
          href={`https://x.com/intent/tweet?in_reply_to=${tweet.id}`}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center hover:text-blue-500 transition-colors cursor-pointer"
        >
          <svg
            className="w-4 h-4 mr-1"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
            />
          </svg>
          {tweet.replyCount || 0}
        </a>
        <a
          href={`https://x.com/intent/retweet?tweet_id=${tweet.id}`}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center hover:text-green-500 transition-colors cursor-pointer"
        >
          <svg
            className="w-4 h-4 mr-1"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"
            />
          </svg>
          {tweet.retweetCount || 0}
        </a>
        <a
          href={`https://x.com/intent/like?tweet_id=${tweet.id}`}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center hover:text-red-500 transition-colors cursor-pointer"
        >
          <svg
            className="w-4 h-4 mr-1"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
            />
          </svg>
          {tweet.likeCount || 0}
        </a>
        <a
          href={`https://x.com/i/bookmarks/add/${tweet.id}`}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center hover:text-blue-400 transition-colors cursor-pointer"
        >
          <svg
            className="w-4 h-4 mr-1"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
            />
          </svg>
          {tweet.bookmarkCount || 0}
        </a>
      </div>
    </div>
  );
}

// Helper functions for formatting
function formatTwitterDate(date: Date): string {
  const timeString = date.toLocaleTimeString([], {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });
  const monthNames = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec",
  ];
  const month = monthNames[date.getMonth()];
  const day = date.getDate();
  const year = date.getFullYear();

  return `${timeString} · ${month} ${day}, ${year}`;
}

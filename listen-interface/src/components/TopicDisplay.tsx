import { useTranslation } from "react-i18next";
import { z } from "zod";

const TypeInteractionsSchema = z.record(z.string(), z.number());

const SentimentDetailSchema = z.object({
  positive: z.number(),
  neutral: z.number(),
  negative: z.number(),
});

const TypesSentimentDetailSchema = z.record(z.string(), SentimentDetailSchema);

export const TopicSchema = z.object({
  topic: z.object({
    topic: z.string(),
    title: z.string(),
    topic_rank: z.number(),
    related_topics: z.array(z.string()),
    types_count: z.record(z.string(), z.number()),
    types_interactions: TypeInteractionsSchema,
    types_sentiment: z.record(z.string(), z.number()),
    types_sentiment_detail: TypesSentimentDetailSchema,
    interactions_24h: z.number(),
    num_contributors: z.number(),
    num_posts: z.number(),
    categories: z.array(z.string()),
    trend: z.enum(["up", "down", "stable"]),
  }),
});

export type Topic = z.infer<typeof TopicSchema>;

export const TopicDisplay = ({ topic }: { topic: Topic }) => {
  const { t } = useTranslation();
  return (
    <div className="border border-[#2D2D2D] rounded-lg p-4 bg-black/40 backdrop-blur-sm">
      {/* Header Section */}
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-xl font-bold">{topic.topic.title}</h2>
          <div className="text-sm text-gray-400">
            {t("topic_display.rank")} #{topic.topic.topic_rank} •{" "}
            {topic.topic.categories.join(", ")}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <div
            className={`text-sm ${
              topic.topic.trend === "up" ? "text-green-500" : "text-red-500"
            }`}
          >
            {t("topic_display.trend")} {topic.topic.trend === "up" ? "↑" : "↓"}
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
        <div className="p-3 border border-[#2D2D2D] rounded-lg">
          <div className="text-sm text-gray-400">
            {t("topic_display.interactions24h")}
          </div>
          <div className="text-lg font-bold">
            {topic.topic.interactions_24h.toLocaleString()}
          </div>
        </div>
        <div className="p-3 border border-[#2D2D2D] rounded-lg">
          <div className="text-sm text-gray-400">
            {t("topic_display.contributors")}
          </div>
          <div className="text-lg font-bold">
            {topic.topic.num_contributors.toLocaleString()}
          </div>
        </div>
        <div className="p-3 border border-[#2D2D2D] rounded-lg">
          <div className="text-sm text-gray-400">
            {t("topic_display.posts")}
          </div>
          <div className="text-lg font-bold">
            {topic.topic.num_posts.toLocaleString()}
          </div>
        </div>
        <div className="p-3 border border-[#2D2D2D] rounded-lg">
          <div className="text-sm text-gray-400">
            {t("topic_display.sentiment")}
          </div>
          <div className="text-lg font-bold">
            {Math.round(
              (topic.topic.types_sentiment_detail.tweet.positive /
                (topic.topic.types_sentiment_detail.tweet.positive +
                  topic.topic.types_sentiment_detail.tweet.negative)) *
                100
            )}
            %
            <span className="text-sm text-gray-400">
              {" "}
              {t("topic_display.positive")}
            </span>
          </div>
        </div>
      </div>

      {/* Content Types */}
      <div className="border-t border-[#2D2D2D] pt-4">
        <h3 className="text-sm font-medium mb-2">
          {t("topic_display.contentDistribution")}
        </h3>
        <div className="space-y-2">
          {Object.entries(topic.topic.types_count).map(([type, count]) => (
            <div key={type} className="flex items-center justify-between">
              <div className="text-sm">{type}</div>
              <div className="text-sm">
                {count.toLocaleString()} (
                {Math.round((count / topic.topic.num_posts) * 100)}%)
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Related Topics */}
      <div className="border-t border-[#2D2D2D] mt-4 pt-4">
        <h3 className="text-sm font-medium mb-2">
          {t("topic_display.relatedTopics")}
        </h3>
        <div className="flex flex-wrap gap-2">
          {topic.topic.related_topics.slice(0, 8).map((relatedTopic) => (
            <span
              key={relatedTopic}
              className="px-2 py-1 text-sm bg-[#2D2D2D] rounded-full"
            >
              {relatedTopic}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
};

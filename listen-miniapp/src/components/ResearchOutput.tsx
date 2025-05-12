import { ChatMessage } from "./ChatMessage";

function embedResearchAnchors(message: string): string {
  // First, temporarily replace escaped underscores in usernames
  // This handles cases like @Felixxx\_on\_sol/1898668765270909366
  let processableMessage = message;

  // Replace escaped underscores with a special placeholder
  const placeholderChar = "§";
  processableMessage = processableMessage.replace(/\\\_/g, placeholderChar);

  // Create a regex that can match Twitter references with both normal and escaped underscores
  const twitterRefRegex = /@([a-zA-Z0-9_§]+)\/(\d+)/g;
  const references: { [key: string]: number } = {};
  let refCount = 0;

  // First pass: collect all references and assign numbers
  let match;
  while ((match = twitterRefRegex.exec(processableMessage)) !== null) {
    // Get the username and tweet ID
    let username = match[1];
    const tweetId = match[2];

    // Convert the placeholder back to underscore for storage and URL creation
    username = username.replace(new RegExp(placeholderChar, "g"), "_");

    const refKey = `${username}/${tweetId}`;

    if (!references[refKey]) {
      refCount++;
      references[refKey] = refCount;
    }
  }

  // Also collect references without @ symbol (like in parentheses)
  const plainRefRegex = /\(([a-zA-Z0-9_§]+)\/(\d+)\)/g;
  while ((match = plainRefRegex.exec(processableMessage)) !== null) {
    let username = match[1];
    const tweetId = match[2];

    // Skip if it's not likely a Twitter reference
    if (username.includes("/") || tweetId.length < 10) {
      continue;
    }

    // Convert the placeholder back to underscore
    username = username.replace(new RegExp(placeholderChar, "g"), "_");

    const refKey = `${username}/${tweetId}`;

    if (!references[refKey]) {
      refCount++;
      references[refKey] = refCount;
    }
  }

  // Also collect references in comma-separated lists
  const commaRefRegex = /([a-zA-Z0-9_§]+)\/(\d+)(?=,|\s|\))/g;
  while ((match = commaRefRegex.exec(processableMessage)) !== null) {
    let username = match[1];
    const tweetId = match[2];

    // Skip if it's not likely a Twitter reference
    if (username.includes("/") || tweetId.length < 10) {
      continue;
    }

    // Convert the placeholder back to underscore
    username = username.replace(new RegExp(placeholderChar, "g"), "_");

    const refKey = `${username}/${tweetId}`;

    if (!references[refKey]) {
      refCount++;
      references[refKey] = refCount;
    }
  }

  // Special case for comma-separated references inside parentheses
  const specialCaseRegex =
    /\(([a-zA-Z0-9_§]+)\/(\d+),\s*([a-zA-Z0-9_§]+)\/(\d+)\)/g;
  let specialMatch;
  while ((specialMatch = specialCaseRegex.exec(processableMessage)) !== null) {
    // First reference
    let username1 = specialMatch[1];
    const tweetId1 = specialMatch[2];

    // Second reference
    let username2 = specialMatch[3];
    const tweetId2 = specialMatch[4];

    // Process first reference
    if (!username1.includes("/") && tweetId1.length >= 10) {
      username1 = username1.replace(new RegExp(placeholderChar, "g"), "_");
      const refKey1 = `${username1}/${tweetId1}`;

      if (!references[refKey1]) {
        refCount++;
        references[refKey1] = refCount;
      }
    }

    // Process second reference
    if (!username2.includes("/") && tweetId2.length >= 10) {
      username2 = username2.replace(new RegExp(placeholderChar, "g"), "_");
      const refKey2 = `${username2}/${tweetId2}`;

      if (!references[refKey2]) {
        refCount++;
        references[refKey2] = refCount;
      }
    }
  }

  // Second pass: replace all references with links
  // We need to handle both normal and escaped versions

  // For each reference, create both the normal and escaped patterns
  for (const refKey of Object.keys(references)) {
    const [username, tweetId] = refKey.split("/");
    const refNumber = references[refKey];
    const twitterUrl = `https://x.com/${username}/status/${tweetId}`;
    const link = `<a href="${twitterUrl}" target="_blank" rel="noopener noreferrer">[${refNumber}]</a>`;

    // Create escaped username pattern (with \_ instead of _)
    const escapedUsername = username.replace(/_/g, "\\_");

    // Replace the escaped version first - with @ symbol
    processableMessage = processableMessage.replace(
      new RegExp(`@${escapedUsername}\\/${tweetId}`, "g"),
      link
    );

    // Replace normal version - with @ symbol
    processableMessage = processableMessage.replace(
      new RegExp(`@${username.replace(/_/g, "[_§]")}\\/${tweetId}`, "g"),
      link
    );

    // Also handle parenthesized versions - with @ symbol
    processableMessage = processableMessage.replace(
      new RegExp(`\\(@${escapedUsername}\\/${tweetId}\\)`, "g"),
      link
    );
    processableMessage = processableMessage.replace(
      new RegExp(`\\(@${username.replace(/_/g, "[_§]")}\\/${tweetId}\\)`, "g"),
      link
    );

    // Handle parenthesized versions - without @ symbol
    processableMessage = processableMessage.replace(
      new RegExp(`\\(${escapedUsername}\\/${tweetId}\\)`, "g"),
      `(${link})`
    );
    processableMessage = processableMessage.replace(
      new RegExp(`\\(${username.replace(/_/g, "[_§]")}\\/${tweetId}\\)`, "g"),
      `(${link})`
    );
  }

  // Restore any remaining escaped underscores
  return processableMessage.replace(new RegExp(placeholderChar, "g"), "\\_");
}

export { embedResearchAnchors }; // Export for testing

export function ResearchOutputDisplay({ message }: { message: string }) {
  const processedMessage = embedResearchAnchors(message);

  return (
    <div className="text-gray-400">
      <ChatMessage message={processedMessage} direction="agent" />
    </div>
  );
}

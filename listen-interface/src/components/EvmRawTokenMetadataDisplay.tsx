import { GtTokenMetadataRaw } from "../types/metadata";
import { Socials } from "./Socials";

export function EvmRawTokenMetadataDisplay({
  metadata,
}: {
  metadata: GtTokenMetadataRaw;
}) {
  return (
    <div className="flex flex-row w-full space-x-8 px-4 py-6">
      {/* Left column: Image, name, symbol, socials */}
      <div className="flex items-start space-x-4 lg:min-w-[300px]">
        {metadata?.image_url && metadata.image_url.startsWith("https://") && (
          <div className="w-16 h-16 sm:w-24 sm:h-24 relative rounded-full overflow-hidden">
            <img
              src={metadata.image_url}
              alt={metadata.name}
              className="w-full h-full object-cover"
            />
          </div>
        )}
        <div>
          <div className="font-medium">
            <div className="flex flex-col">
              <span className="text-xl font-bold text-white mb-1">
                {metadata?.name || "Unknown Token"}
              </span>
              <span className="text-md text-white">
                ${metadata?.symbol || "unknown"}
              </span>
            </div>
          </div>
          <div className="mt-3">
            <Socials
              tokenMetadata={{
                twitter: metadata.twitter_handle,
                telegram: metadata.telegram_handle,
                discord: metadata.discord_url,
                website: metadata.websites?.[0],
              }}
              pubkey={metadata.address}
              chainId={metadata.chain_id}
            />
          </div>
        </div>
      </div>

      {/* Right column: Description */}
      {metadata?.description && (
        <div className="flex-1 text-white whitespace-pre-line flex items-center relative">
          <div
            className="max-h-[200px] overflow-y-auto w-full scrollbar-container 
                        [mask-image:linear-gradient(to_bottom,transparent,black_20px,black_calc(100%-20px),transparent)]"
          >
            <p className="text-sm text-center mx-auto py-4">
              {metadata.description}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

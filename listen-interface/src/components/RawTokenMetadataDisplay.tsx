import { useModal } from "../contexts/ModalContext";
import { TokenMetadataRaw } from "../types/metadata";
import { Socials } from "./Socials";

export function RawTokenMetadataDisplay({
  metadata,
}: {
  metadata: TokenMetadataRaw;
}) {
  const { openChart } = useModal();

  return (
    <div className="flex flex-row w-full space-x-8 px-4 py-6 items-center">
      {/* Left column: Image, name, symbol, socials */}
      <div className="flex items-center space-x-4 lg:min-w-[300px]">
        {metadata?.mpl.ipfs_metadata?.image &&
          metadata.mpl.ipfs_metadata.image.startsWith("https://") && (
            <div className="w-16 h-16 sm:w-24 sm:h-24 relative rounded-full overflow-hidden">
              <img
                src={metadata.mpl.ipfs_metadata.image.replace(
                  "cf-ipfs.com",
                  "ipfs.io"
                )}
                alt={metadata.mpl.name}
                className="w-full h-full object-cover"
              />
            </div>
          )}
        <div>
          <div className="font-medium">
            <div className="flex flex-col">
              <span className="text-xl font-bold text-white mb-1">
                {metadata?.mpl.name || "Unknown Token"}
              </span>
              <span className="text-md text-white">
                ${metadata?.mpl.symbol || "unknown"}
              </span>
            </div>
          </div>
          <div className="mt-3">
            <Socials
              tokenMetadata={metadata ?? null}
              pubkey={metadata.mint}
              openChart={openChart}
            />
          </div>
        </div>
      </div>

      {/* Right column: Description */}
      {metadata?.mpl.ipfs_metadata?.description && (
        <div className="flex-1 text-white whitespace-pre-line flex items-center justify-center relative">
          <div
            className="max-h-[200px] overflow-y-auto w-full scrollbar-container
                        [mask-image:linear-gradient(to_bottom,transparent,black_20px,black_calc(100%-20px),transparent)]"
          >
            <p className="text-sm text-center mx-auto py-4">
              {metadata.mpl.ipfs_metadata.description}
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

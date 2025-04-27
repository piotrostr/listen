export function PortfolioSkeleton() {
  return (
    <div className="h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container pb-16 md:pb-0 p-4">
      {/* Summary Section - matching PortfolioSummary exactly */}
      <div className="flex flex-col justify-center p-10 gap-7 w-full pt-8">
        {/* Balance Skeleton */}
        <span className="font-space-grotesk font-medium text-[45px] leading-4 text-white text-center">
          <div className="h-12 w-48 bg-[#2D2D2D] rounded animate-pulse mx-auto" />
        </span>
        {/* Action Buttons Skeleton - matching TileButton layout */}
        <div className="flex flex-row items-center gap-3 justify-center mt-2">
          {[1, 2, 3].map((index) => (
            <div
              key={index}
              className="w-8 h-8 rounded bg-[#2D2D2D] animate-pulse"
            />
          ))}
        </div>
      </div>

      {/* Portfolio Items */}
      <div className="flex-1 space-y-2">
        {[1, 2, 3, 4].map((index) => (
          <div
            key={index}
            className="p-3 sm:p-4 bg-[#2d2d2d]/20 transition-colors rounded-2xl"
          >
            <div className="flex justify-between items-start">
              <div className="flex items-center gap-3">
                <div className="relative">
                  <div className="w-12 h-12 rounded-full bg-[#2D2D2D] animate-pulse" />
                  <div className="absolute top-1 -left-1 z-10">
                    <div className="w-4 h-4 rounded-full bg-[#2D2D2D] animate-pulse" />
                  </div>
                </div>
                <div>
                  <h3 className="font-bold flex items-center gap-2">
                    <div className="h-5 w-24 bg-[#2D2D2D] rounded animate-pulse" />
                  </h3>
                  <p className="text-sm text-gray-400 mt-1">
                    <div className="h-4 w-20 bg-[#2D2D2D] rounded animate-pulse" />
                  </p>
                </div>
              </div>
              <div className="text-right">
                <div className="flex justify-end gap-2">
                  <div className="text-right">
                    <div className="h-5 w-24 bg-[#2D2D2D] rounded animate-pulse mb-1" />
                    <div className="h-4 w-16 bg-[#2D2D2D] rounded animate-pulse" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

export function PortfolioSkeleton() {
  return (
    <div className="h-full font-mono">
      {/* Header section - matches Portfolio.tsx layout */}
      <div className="flex lg:flex-row flex-col lg:justify-between lg:items-center p-4 lg:mt-3 lg:mb-3">
        <div className="h-7 w-28 bg-purple-500/20 rounded animate-pulse lg:mb-0 mb-2" />
        <div className="flex lg:flex-row flex-col lg:items-center gap-2">
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded-full bg-purple-500/20 animate-pulse" />
            <div className="h-5 w-24 bg-purple-500/20 rounded animate-pulse" />
            <div className="h-4 w-4 bg-purple-500/20 rounded animate-pulse" />
          </div>
          <div className="flex items-center gap-2">
            <div className="w-4 h-4 rounded-full bg-purple-500/20 animate-pulse" />
            <div className="h-5 w-24 bg-purple-500/20 rounded animate-pulse" />
            <div className="h-4 w-4 bg-purple-500/20 rounded animate-pulse" />
          </div>
        </div>
      </div>

      {/* Main content area */}
      <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-purple-500/30 scrollbar-track-transparent">
        <div className="p-4 pt-0 space-y-4">
          {[1, 2, 3, 4].map((index) => (
            <div
              key={index}
              className="border border-purple-500/30 rounded-lg p-3"
            >
              <div className="flex justify-between items-start mb-2">
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 rounded-full bg-purple-500/20 animate-pulse" />
                  <div>
                    <div className="h-5 w-24 bg-purple-500/20 rounded animate-pulse mb-1" />
                    <div className="h-4 w-16 bg-purple-500/20 rounded animate-pulse" />
                  </div>
                </div>
                <div className="text-right">
                  <div className="h-5 w-20 bg-purple-500/20 rounded animate-pulse mb-1" />
                  <div className="h-4 w-24 bg-purple-500/20 rounded animate-pulse" />
                </div>
              </div>
              <div className="h-4 w-32 bg-purple-500/20 rounded animate-pulse mt-2" />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

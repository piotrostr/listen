export function PortfolioSkeleton() {
  return (
    <div className="h-full font-mono overflow-y-auto scrollbar-thin scrollbar-thumb-[#2D2D2D] scrollbar-track-transparent scrollable-container">
      <div className="flex-1">
        <div className="p-4 pt-0 space-y-4">
          {[1, 2, 3, 4].map((index) => (
            <div key={index} className="border border-[#2D2D2D] rounded-lg p-3">
              <div className="flex justify-between items-start mb-2">
                <div className="flex items-center gap-3">
                  <div className="w-8 h-8 rounded-full bg-[#2D2D2D] animate-pulse" />
                  <div>
                    <div className="h-5 w-24 bg-[#2D2D2D] rounded animate-pulse mb-1" />
                    <div className="h-4 w-16 bg-[#2D2D2D] rounded animate-pulse" />
                  </div>
                </div>
                <div className="text-right">
                  <div className="flex items-center gap-2">
                    <div className="h-5 w-20 bg-[#2D2D2D] rounded animate-pulse mb-1" />
                  </div>
                </div>
              </div>
              <div className="flex justify-between items-center">
                <div className="h-4 w-32 bg-[#2D2D2D] rounded animate-pulse" />
                <div className="flex gap-2">
                  <div className="w-6 h-6 rounded bg-[#2D2D2D] animate-pulse" />
                  <div className="w-6 h-6 rounded bg-[#2D2D2D] animate-pulse" />
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

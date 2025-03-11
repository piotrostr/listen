import { createContext, ReactNode, useContext, useState } from "react";
import { createPortal } from "react-dom";
import { Chart } from "../components/Chart";

interface ModalContextType {
  openChart: (mint: string) => void;
  closeChart: () => void;
}

const ModalContext = createContext<ModalContextType | null>(null);

export function ModalProvider({ children }: { children: ReactNode }) {
  const [chartMint, setChartMint] = useState<string | null>(null);

  const openChart = (mint: string) => setChartMint(mint);
  const closeChart = () => setChartMint(null);

  return (
    <ModalContext.Provider value={{ openChart, closeChart }}>
      {children}
      {chartMint &&
        createPortal(
          <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div className="fixed inset-0 bg-[#151518]/60 backdrop-blur-sm pointer-events-none" />
            <div className="relative bg-[#151518]/40  w-[90vw] h-[80vh] rounded-xl p-6 backdrop-blur-sm pointer-events-auto">
              <button
                onClick={closeChart}
                className="absolute top-4 right-4 text-purple-300/70 hover:text-purple-100 transition-colors"
              >
                ✕
              </button>
              <div className="w-full h-full">
                <Chart mint={chartMint} />
              </div>
            </div>
            <div className="fixed inset-0 z-[-1]" onClick={closeChart} />
          </div>,
          document.body
        )}
    </ModalContext.Provider>
  );
}

export const useModal = () => {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error("useModal must be used within a ModalProvider");
  }
  return context;
};

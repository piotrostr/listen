import listenLoading from "../assets/listen-loading.gif";

export const ThinkingIndicator = () => (
  <div className="h-4 w-4 rounded-full animate-[spherePulse_3s_ease-in-out_infinite] shadow-lg relative">
    <div className="absolute inset-0 rounded-full opacity-70 blur-[1px] animate-colorChange"></div>
  </div>
);

export const _ThinkingIndicator = () => (
  <div>
    <img src={listenLoading} className="w-8 animate-fast" />
  </div>
);

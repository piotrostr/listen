import { createFileRoute } from "@tanstack/react-router";
import { PriceUpdates } from "../components/PriceUpdates";

export const Route = createFileRoute("/screener")({
  component: ScreenerPage,
});

function ScreenerPage() {
  return (
    <div className="flex-1 overflow-hidden">
      <PriceUpdates />
    </div>
  );
}

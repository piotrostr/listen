import { createFileRoute } from "@tanstack/react-router";
import { PriceUpdates } from "../components/PriceUpdates";

export const Route = createFileRoute("/screener")({
  component: ScreenerPage,
});

function ScreenerPage() {
  return (
    <div className="max-w-7xl mx-auto px-4">
      <PriceUpdates />
    </div>
  );
}

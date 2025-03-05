import { createFileRoute } from "@tanstack/react-router";
import { PriceUpdates } from "../components/PriceUpdates";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
  return (
    <div className="flex-1 overflow-hidden">
      <PriceUpdates />
    </div>
  );
}

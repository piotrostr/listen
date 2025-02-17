import { createFileRoute } from "@tanstack/react-router";
import { Portfolio } from "../components/Portfolio";

export const Route = createFileRoute("/portfolio")({
  component: PortfolioPage,
});

function PortfolioPage() {
  return (
    <div className="max-w-7xl mx-auto px-4">
      <Portfolio />
    </div>
  );
}

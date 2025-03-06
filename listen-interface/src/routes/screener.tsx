import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/screener")({
  component: ScreenerPage,
  beforeLoad: () => {
    throw redirect({ to: "/" });
  },
});

function ScreenerPage() {
  return null;
}

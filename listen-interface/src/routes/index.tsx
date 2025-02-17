import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  component: Index,
  beforeLoad: () => {
    throw redirect({ to: "/screener" });
  },
});

function Index() {
  return null;
}

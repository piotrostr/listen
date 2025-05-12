import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/portfolio")({
  beforeLoad: ({ search }) => {
    // Redirect to the root route while preserving all search parameters
    throw redirect({
      to: "/",
      search: search,
      replace: true,
    });
  },
});

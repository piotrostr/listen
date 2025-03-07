import { createRootRoute } from "@tanstack/react-router";
import { z } from "zod";
import { Chat } from "../components/Chat";
import { GettingStarted } from "../components/GettingStarted";
import { Layout } from "../components/Layout";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";

const SearchParamsSchema = z.object({
  chatId: z.string().optional(),
  new: z.boolean().optional(),
  shared: z.boolean().optional(),
});

export const Route = createRootRoute({
  component: RootComponent,
  validateSearch: (search: Record<string, unknown>) => {
    return SearchParamsSchema.parse(search);
  },
});

function RootComponent() {
  const { isAuthenticated, ready } = useIsAuthenticated();

  if (!ready) {
    return (
      <Layout>
        <></>
      </Layout>
    );
  }

  return <Layout>{isAuthenticated ? <Chat /> : <GettingStarted />}</Layout>;
}

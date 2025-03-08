import { createRootRoute, Outlet } from "@tanstack/react-router";
import { Layout } from "../components/Layout";

export const Route = createRootRoute({
  component: RootComponent,
});

function RootComponent() {
  return (
    <Layout>
      <Outlet />
    </Layout>
  );
}

import { createRootRoute, Outlet } from "@tanstack/react-router";
import { useEffect } from "react";
import { GettingStarted } from "../components/GettingStarted";
import { Layout } from "../components/Layout";
import { useIsAuthenticated } from "../hooks/useIsAuthenticated";
import { checkAppVersion } from "../utils/version";

export const Route = createRootRoute({
  component: RootComponent,
  beforeLoad: async ({ location }) => {
    if (location.pathname === "/") {
      return;
    }
  },
});

function RootComponent() {
  const { isAuthenticated, ready } = useIsAuthenticated();

  // Check for new version on component mount
  useEffect(() => {
    // This will automatically reload the page if a new version is detected
    checkAppVersion();
  }, []);

  if (!ready) {
    return (
      <Layout>
        <></>
      </Layout>
    );
  }

  return <Layout>{isAuthenticated ? <Outlet /> : <GettingStarted />}</Layout>;
}

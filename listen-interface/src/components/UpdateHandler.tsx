import { useEffect, useState } from "react";
// @ts-ignore - Virtual module provided by vite-plugin-pwa
import { useRegisterSW } from "virtual:pwa-register/react";

// Styles for the update notification
const updateStyles = {
  container: {
    position: "fixed",
    bottom: "20px",
    right: "20px",
    backgroundColor: "#A855F7",
    color: "white",
    padding: "12px 20px",
    borderRadius: "8px",
    boxShadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
    zIndex: 1000,
    display: "flex",
    flexDirection: "column",
    gap: "8px",
    maxWidth: "320px",
  },
  title: {
    fontWeight: "bold",
    fontSize: "16px",
  },
  message: {
    fontSize: "14px",
  },
  button: {
    backgroundColor: "white",
    color: "#A855F7",
    border: "none",
    padding: "8px 16px",
    borderRadius: "4px",
    cursor: "pointer",
    fontWeight: "bold",
    fontSize: "14px",
    marginTop: "8px",
  },
};

// Define types for service worker registration
interface ServiceWorkerRegistration {
  update(): Promise<void>;
}

export const UpdateHandler = () => {
  const [showReload, setShowReload] = useState(false);

  // Register service worker and handle updates
  const {
    needRefresh: [_needRefresh, setNeedRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegistered(r: ServiceWorkerRegistration | undefined) {
      console.log("SW registered:", r);

      // Check for updates every 60 minutes
      setInterval(
        () => {
          r?.update().catch(console.error);
        },
        60 * 60 * 1000
      );
    },
    onRegisterError(error: Error) {
      console.error("SW registration error:", error);
    },
    onNeedRefresh() {
      // New content is available, show update notification
      setShowReload(true);
      setNeedRefresh(true);
    },
  });

  const handleReload = () => {
    updateServiceWorker(true);
    window.location.reload();
  };

  useEffect(() => {
    // Force check for updates on component mount
    navigator.serviceWorker?.getRegistrations().then((regs) => {
      for (const reg of regs) {
        reg.update().catch(console.error);
      }
    });
  }, []);

  if (!showReload) return null;

  return (
    <div style={updateStyles.container as React.CSSProperties}>
      <div style={updateStyles.title}>Update Available</div>
      <div style={updateStyles.message}>
        A new version of Listen is available. Please refresh to get the latest
        features and improvements.
      </div>
      <button style={updateStyles.button} onClick={handleReload}>
        Update Now
      </button>
    </div>
  );
};

export default UpdateHandler;

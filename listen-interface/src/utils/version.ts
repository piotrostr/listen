// This will only change when you build and deploy a new version
// It's a string constant, not a function that gets called each time
export const BUILD_ID = "__BUILD_TIME__" + process.env.NODE_ENV;

// Create a consistent key name for localStorage
const VERSION_KEY = "listen-app-version";

// A flag to avoid checking more than once per session
let hasCheckedThisSession = false;

/**
 * Simple function to force a page reload when a new version is deployed
 * Call this at app startup, and it will reload if necessary
 */
export function checkAppVersion(): void {
  // Only check once per session to avoid any possibility of reload loops
  if (hasCheckedThisSession) {
    return;
  }

  hasCheckedThisSession = true;
  const lastVersion = localStorage.getItem(VERSION_KEY);

  // If this is the first time, just save the current version
  if (!lastVersion) {
    localStorage.setItem(VERSION_KEY, BUILD_ID);
    return;
  }

  // If we have a different version than what was saved, update and reload
  if (lastVersion !== BUILD_ID) {
    console.log("New app version detected, refreshing...");
    localStorage.setItem(VERSION_KEY, BUILD_ID);

    // Use setTimeout to ensure the localStorage is updated before reload
    setTimeout(() => {
      window.location.reload();
    }, 100);
  }
}

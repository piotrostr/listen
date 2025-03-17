import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

interface MobileContextType {
  isMobile: boolean;
  isVerySmallScreen: boolean;
  isIOS: boolean;
}

const MobileContext = createContext<MobileContextType>({
  isMobile: false,
  isVerySmallScreen: false,
  isIOS: false,
});

export function MobileProvider({ children }: { children: ReactNode }) {
  const [isMobile, setIsMobile] = useState(false);
  const [isVerySmallScreen, setIsVerySmallScreen] = useState(false);
  const [isIOS, setIsIOS] = useState(false);

  useEffect(() => {
    // Detect if device is mobile or very small screen
    const checkScreenSize = () => {
      setIsMobile(window.innerWidth < 600);
      setIsVerySmallScreen(window.innerWidth < 390);
    };

    // Detect iOS device
    const checkIOS = () => {
      const userAgent = window.navigator.userAgent.toLowerCase();
      setIsIOS(/iphone|ipad|ipod/.test(userAgent));
    };

    checkScreenSize();
    checkIOS();

    window.addEventListener("resize", checkScreenSize);
    return () => window.removeEventListener("resize", checkScreenSize);
  }, []);

  return (
    <MobileContext.Provider value={{ isMobile, isVerySmallScreen, isIOS }}>
      {children}
    </MobileContext.Provider>
  );
}

export function useMobile() {
  return useContext(MobileContext);
}

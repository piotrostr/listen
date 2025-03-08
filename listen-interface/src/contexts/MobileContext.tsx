import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useState,
} from "react";

interface MobileContextType {
  isMobile: boolean;
  isIOS: boolean;
}

const MobileContext = createContext<MobileContextType>({
  isMobile: false,
  isIOS: false,
});

export function MobileProvider({ children }: { children: ReactNode }) {
  const [isMobile, setIsMobile] = useState(false);
  const [isIOS, setIsIOS] = useState(false);

  useEffect(() => {
    // Detect if device is mobile
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 600);
    };

    // Detect iOS device
    const checkIOS = () => {
      const userAgent = window.navigator.userAgent.toLowerCase();
      setIsIOS(/iphone|ipad|ipod/.test(userAgent));
    };

    checkMobile();
    checkIOS();

    window.addEventListener("resize", checkMobile);
    return () => window.removeEventListener("resize", checkMobile);
  }, []);

  return (
    <MobileContext.Provider value={{ isMobile, isIOS }}>
      {children}
    </MobileContext.Provider>
  );
}

export function useMobile() {
  return useContext(MobileContext);
}

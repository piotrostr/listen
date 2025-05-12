import { createContext, ReactNode, useContext, useState } from "react";

type SidebarContextType = {
  setIsSidebarOpen: (open: boolean) => void;
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
  isDropdownOpen: boolean;
  setIsDropdownOpen: (open: boolean) => void;
};

const SidebarContext = createContext<SidebarContextType>({
  setIsSidebarOpen: () => {},
  isSidebarOpen: false,
  toggleSidebar: () => {},
  isDropdownOpen: false,
  setIsDropdownOpen: () => {},
});

export const SidebarProvider = ({ children }: { children: ReactNode }) => {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);

  const toggleSidebar = () => setIsSidebarOpen((prev) => !prev);

  return (
    <SidebarContext.Provider
      value={{
        isSidebarOpen,
        setIsSidebarOpen,
        toggleSidebar,
        isDropdownOpen,
        setIsDropdownOpen,
      }}
    >
      {children}
    </SidebarContext.Provider>
  );
};

export const useSidebar = () => {
  const context = useContext(SidebarContext);

  if (context === undefined) {
    throw new Error("useSidebar must be used within a SidebarProvider");
  }

  return context;
};

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

const MenuItem = ({ href, children, external }) => {
  return (
    <motion.a
      href={href}
      target={external ? "_blank" : "_self"}
      rel={external ? "noopener noreferrer" : ""}
      className="relative group"
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
    >
      <span className="text-gray-300 hover:text-white transition-colors duration-200 font-medium flex items-center">
        {children}
      </span>
      <motion.div
        className="absolute bottom-0 left-0 w-0 h-0.5 bg-gradient-to-r from-blue-400 to-purple-400 group-hover:w-full transition-all duration-300"
        whileHover={{ width: "100%" }}
      />
    </motion.a>
  );
};

const MobileMenuItem = ({ href, children, external, onClick }) => {
  return (
    <motion.a
      href={href}
      target={external ? "_blank" : "_self"}
      rel={external ? "noopener noreferrer" : ""}
      onClick={onClick}
      className="block w-full px-4 py-3 text-base font-medium text-gray-300 hover:text-white transition-colors duration-200"
      whileHover={{ backgroundColor: "rgba(55, 65, 81, 0.5)" }}
      whileTap={{ scale: 0.98 }}
    >
      <div className="flex items-center justify-between">
        {children}
        {external && (
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            className="ml-2"
            fill="currentColor"
          >
            <path d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z" />
          </svg>
        )}
      </div>
    </motion.a>
  );
};

export const Header = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  return (
    <motion.nav
      initial={{ y: -100 }}
      animate={{ y: 0 }}
      transition={{ type: "spring", stiffness: 100, damping: 20 }}
      className="fixed top-0 left-0 right-0 z-50"
    >
      <div className="bg-gray-900/80 backdrop-blur border-b border-gray-800/50 shadow-lg">
        <div className="max-w-6xl mx-auto px-4">
          <div className="flex justify-between items-center h-16">
            {/* Left side */}
            <div className="flex items-center space-x-4">
              <motion.button
                className="md:hidden p-2 hover:bg-gray-800/50 rounded-lg transition-colors duration-200"
                aria-label="Toggle navigation"
                onClick={toggleMenu}
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.9 }}
              >
                <svg
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  className="w-6 h-6"
                >
                  <path
                    strokeLinecap="round"
                    strokeWidth="2"
                    d={isMenuOpen ? "M6 18L18 6M6 6l12 12" : "M4 6h16M4 12h16M4 18h16"}
                  />
                </svg>
              </motion.button>

              <motion.a
                href="/"
                className="flex items-center space-x-2 group"
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
              >
                <motion.img
                  src="/listen.svg"
                  alt="Logo"
                  className="w-8 h-8"
                  whileHover={{ rotate: 180 }}
                  transition={{ duration: 0.3 }}
                />
                <motion.span 
                  className="font-bold text-xl bg-gradient-to-r from-blue-400 to-purple-400 bg-clip-text text-transparent"
                >
                  listen-rs
                </motion.span>
              </motion.a>
            </div>

            {/* Desktop Menu */}
            <div className="hidden md:flex items-center space-x-8">
              <MenuItem href="https://docs.listen-rs.com">
                Documentation
              </MenuItem>
              <MenuItem href="https://github.com/piotrostr/listen" external>
                GitHub
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  className="ml-1"
                  fill="currentColor"
                >
                  <path d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z" />
                </svg>
              </MenuItem>
            </div>
          </div>

          {/* Mobile Menu */}
          <AnimatePresence>
            {isMenuOpen && (
              <motion.div
                initial={{ opacity: 0, height: 0 }}
                animate={{ opacity: 1, height: "auto" }}
                exit={{ opacity: 0, height: 0 }}
                transition={{ duration: 0.2 }}
                className="md:hidden overflow-hidden"
              >
                <motion.div
                  initial={{ x: -20, opacity: 0 }}
                  animate={{ x: 0, opacity: 1 }}
                  transition={{ delay: 0.1 }}
                  className="py-2 space-y-1"
                >
                  <MobileMenuItem 
                    href="https://docs.listen-rs.com"
                    onClick={() => setIsMenuOpen(false)}
                  >
                    Documentation
                  </MobileMenuItem>
                  <MobileMenuItem 
                    href="https://github.com/piotrostr/listen"
                    external
                    onClick={() => setIsMenuOpen(false)}
                  >
                    GitHub
                  </MobileMenuItem>
                </motion.div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </motion.nav>
  );
};
import { useState } from "react";

export const Header = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const toggleMenu = () => {
    setIsMenuOpen(!isMenuOpen);
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 bg-gray-900/80 backdrop-blur border-b border-gray-800">
      <div className="max-w-6xl mx-auto px-4">
        <div className="flex justify-between items-center h-16">
          {/* Left side */}
          <div className="flex items-center space-x-4">
            <button
              className="md:hidden p-2"
              aria-label="Toggle navigation"
              onClick={toggleMenu}
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
                  d="M4 6h16M4 12h16M4 18h16"
                />
              </svg>
            </button>

            <a href="/" className="flex items-center space-x-2">
              <img
                src="/listen-more.png"
                alt="Logo"
                className="w-8 h-8 rounded"
              />
              <span className="font-bold text-xl">listen-rs</span>
            </a>
          </div>

          {/* Right side */}
          <div className="flex items-center space-x-4 hidden md:flex">
            <div className="items-center space-x-4">
              <a
                href="https://docs.listen-rs.com"
                className="text-gray-300 hover:text-white"
              >
                Documentation
              </a>
            </div>
            <a
              href="https://github.com/piotrostr/listen"
              target="_blank"
              rel="noopener noreferrer"
              className="text-gray-300 hover:text-white flex items-center"
            >
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
            </a>
          </div>
        </div>

        {/* Mobile menu */}
        {isMenuOpen && (
          <div className="md:hidden">
            <div className="px-2 pt-2 pb-3 space-y-1 bg-gray-900/80 backdrop-blur">
              <a
                href="https://docs.listen-rs.com"
                className="block px-3 py-2 text-base font-medium text-gray-300 hover:text-white hover:bg-gray-700 rounded-md"
              >
                Documentation
              </a>
            </div>
            <div className="px-2 pt-2 pb-3 space-y-1 bg-gray-900/80 backdrop-blur mb-2 mt-2">
              <a
                href="https://github.com/piotrostr/listen"
                target="_blank"
                rel="noopener noreferrer"
                className="block px-3 py-2 text-base font-medium text-gray-300 hover:text-white hover:bg-gray-700 rounded-md flex flex-row items-center"
              >
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
              </a>
            </div>
          </div>
        )}
      </div>
    </nav>
  );
};

/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        mono: ["Space Grotesk", "monospace"],
        'space-grotesk': ['Space Grotesk', 'sans-serif'],
      },
      animation: {
        blob: "blob 7s infinite",
        'colorChange': 'colorChange 2s ease-in-out infinite',
      },
      keyframes: {
        blob: {
          "0%": {
            transform: "translate(0px, 0px) scale(1)",
          },
          "33%": {
            transform: "translate(30px, -50px) scale(1.1)",
          },
          "66%": {
            transform: "translate(-20px, 20px) scale(0.9)",
          },
          "100%": {
            transform: "translate(0px, 0px) scale(1)",
          },
        },
        colorChange: {
          '0%, 100%': { backgroundColor: 'rgba(253, 152, 162, 1)' },
          '25%': { backgroundColor: 'rgba(251, 38, 113, 1)' },
          '50%': { backgroundColor: 'rgba(164, 44, 205, 1)' },
          '75%': { backgroundColor: 'rgba(127, 74, 251, 1)' }
        },
      },
      colors: {
        black: "#151518",
      },
    },
  },
  plugins: [
    require("tailwind-scrollbar"),
    require('tailwind-scrollbar-hide')
  ],
};

/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        mono: ["Space Grotesk", "monospace"],
        'space-grotesk': ['Space Grotesk', 'sans-serif'],
        'dm-sans': ['DM Sans', 'sans-serif'],
        'work-sans': ['Work Sans', 'sans-serif'],
        'work-sans-chat': ['Work Sans', 'sans-serif'],
      },
      backgroundImage: {
        'text-gradient': 'linear-gradient(180deg, rgba(255, 255, 255, 0.8) 60%, #151518 100%)',
      },
      letterSpacing: {
        'chat': '-0.04em',
      },
      textStyles: {
        'chat': {
          fontFamily: 'Work Sans',
          fontSize: '16px',
          fontWeight: '400',
          lineHeight: '150%',
          letterSpacing: '-0.04em',
          verticalAlign: 'middle',
          backgroundImage: 'linear-gradient(180deg, rgba(255, 255, 255, 0.8) 60%, #151518 100%)',
          backgroundClip: 'text',
          textFillColor: 'transparent',
        },
      },
      fontSize: {
        'work-sans-chat': ['16px', {
          lineHeight: '150%',
          letterSpacing: '-0.04em',
          fontWeight: '400',
        }],
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
        'pump-green': '#8DFC63',
        'pump-green-bg': '#8DFC631A',
        'pump-red': '#F72777',
        'pump-red-bg': '#F727771A',
      },
    },
  },
  plugins: [
    require("tailwind-scrollbar"),
    require('tailwind-scrollbar-hide'),
    function({ addUtilities }) {
      addUtilities({
        '.font-work-sans-chat': {
          fontFamily: 'Work Sans, sans-serif',
          fontSize: '16px',
          fontWeight: '400',
          lineHeight: '150%',
          letterSpacing: '-0.04em',
          verticalAlign: 'middle',
          color: '#D0D0D1',
        },
      })
    }
  ],
};

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        studio: {
          bg: '#0f172a',
          surface: '#1e293b',
          border: '#334155',
          accent: '#38bdf8',
          accentHover: '#0ea5e9',
          success: '#10b981',
          warning: '#f59e0b',
          error: '#ef4444',
          text: '#f8fafc',
          muted: '#94a3b8',
        }
      }
    },
  },
  plugins: [],
}

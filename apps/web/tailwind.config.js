/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        sans: ['Geist', 'system-ui', 'sans-serif'],
        mono: ['Geist Mono', 'JetBrains Mono', 'monospace'],
      },
      colors: {
        vigil: {
          bg: '#030712',
          bg2: '#0b0f19',
          card: '#111827',
          border: '#1f2937',
          accent: '#f59e0b',
          'accent-glow': 'rgba(245, 158, 11, 0.35)',
          green: '#10b981',
          red: '#ef4444',
          text: '#f8fafc',
          muted: '#94a3b8',
          dim: '#475569',
        }
      },
      animation: {
        'pulse-dot': 'pulse-dot 2s infinite',
        'marquee': 'marquee 30s linear infinite',
      },
      keyframes: {
        'pulse-dot': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.3' },
        },
        'marquee': {
          '0%': { transform: 'translateX(0%)' },
          '100%': { transform: 'translateX(-50%)' },
        },
      },
    },
  },
  plugins: [],
}

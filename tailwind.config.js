/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Modern dark theme with vibrant accents
        editor: {
          bg: '#0A0A0F',          // Deep space black
          surface: '#13131A',      // Card background
          elevated: '#1A1A24',     // Elevated surfaces
          border: '#2A2A35',       // Borders
          hover: '#252530',        // Hover states
        },
        accent: {
          primary: '#8B5CF6',      // Vibrant purple (like Cursor)
          secondary: '#3B82F6',    // Rich blue
          success: '#10B981',      // Fresh green
          warning: '#F59E0B',      // Warm amber
          danger: '#EF4444',       // Bold red
        },
        text: {
          primary: '#F1F5F9',      // Pure white text
          secondary: '#94A3B8',    // Muted text
          tertiary: '#64748B',     // Subtle text
          disabled: '#475569',     // Disabled text
        },
      },
      boxShadow: {
        'glow-sm': '0 0 10px rgba(139, 92, 246, 0.3)',
        'glow-md': '0 0 20px rgba(139, 92, 246, 0.4)',
        'glow-lg': '0 0 30px rgba(139, 92, 246, 0.5)',
        'glass': '0 8px 32px 0 rgba(0, 0, 0, 0.37)',
      },
      backdropBlur: {
        xs: '2px',
      },
      animation: {
        'fade-in': 'fadeIn 0.2s ease-in-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'scale-in': 'scaleIn 0.2s ease-out',
        'glow': 'glow 2s ease-in-out infinite',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideIn: {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        scaleIn: {
          '0%': { transform: 'scale(0.95)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        glow: {
          '0%, 100%': { opacity: '0.5' },
          '50%': { opacity: '1' },
        },
      },
      fontFamily: {
        sans: [
          'Inter',
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          '"Helvetica Neue"',
          'Arial',
          'sans-serif',
        ],
        mono: [
          'JetBrains Mono',
          'Menlo',
          'Monaco',
          '"Courier New"',
          'monospace',
        ],
      },
    },
  },
  plugins: [],
}

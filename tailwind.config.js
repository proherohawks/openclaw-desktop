/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        claw: {
          bg: "#0a0a0b",
          panel: "#0f0f11",
          surface: "#141416",
          card: "#1a1a1f",
          border: "#1e1e22",
          "border-hover": "#2a2a30",
          "border-focus": "#3d3d48",
          amber: "#f59e0b",
          "amber-light": "#fbbf24",
          green: "#22c55e",
          muted: "#444",
          dim: "#333",
          subtle: "#555",
          text: "#e2e2e2",
          "text-bright": "#f0f0f0",
          "text-muted": "#999",
          "user-bg": "#1c1810",
          "user-border": "#3d2f0a",
          "error-bg": "#1a0f0f",
          "error-border": "#3a1a1a",
          "error-avatar-bg": "#3a1a1a",
          "error-avatar-border": "#5a2a2a",
        },
      },
      fontFamily: {
        mono: ["'JetBrains Mono'", "'Fira Code'", "'Cascadia Code'", "monospace"],
      },
      keyframes: {
        "dot-pulse": {
          "0%, 80%, 100%": { opacity: "0.3", transform: "scale(0.8)" },
          "40%": { opacity: "1", transform: "scale(1)" },
        },
      },
      animation: {
        "dot-pulse": "dot-pulse 1.2s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};

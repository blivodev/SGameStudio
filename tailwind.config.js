export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "rgb(var(--sg-bg) / <alpha-value>)",
        surface: "rgb(var(--sg-surface) / <alpha-value>)",
        surfaceAlt: "rgb(var(--sg-surface-alt) / <alpha-value>)",
        border: "rgb(var(--sg-border) / <alpha-value>)",
        accent: "rgb(var(--sg-accent) / <alpha-value>)",
        accent2: "rgb(var(--sg-accent-2) / <alpha-value>)",
        text: "rgb(var(--sg-text) / <alpha-value>)",
        muted: "rgb(var(--sg-muted) / <alpha-value>)"
      }
    }
  }
};
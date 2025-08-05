import type { Config } from "tailwindcss";
import daisyui from "daisyui";

export default {
  content: ["./src/**/*.tsx"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["var(--font-sans)"],
      },
    },
  },
  plugins: [daisyui],
  daisyui: {
    themes: [
      {
        mytheme: {
          "primary": "#1e40af",
          "secondary": "#9333ea",
          "accent": "#f43f5e",
          "neutral": "#111827",
          "base-100": "#ffffff",
        },
      },
    ],
  },
} satisfies Config;
import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./app/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: "#101922",
        panel: "#17232e",
        line: "#293744",
        mint: "#67e8c1",
      },
    },
  },
  plugins: [],
};

export default config;

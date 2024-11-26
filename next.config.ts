import type { NextConfig } from "next";

const config: NextConfig = {
  output: "export",
  experimental: {
    reactCompiler: true,
  },
  images: {
    unoptimized: true,
  },
};

export default config;

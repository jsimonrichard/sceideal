/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: "http://127.0.0.1:3001/api/:path*", // Proxy to Backend
      },
    ];
  },
  async redirects() {
    return [
      {
        source: "/settings",
        destination: "/settings/account",
        permanent: true,
      },
    ];
  },
};

module.exports = nextConfig;

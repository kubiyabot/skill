/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export', // Static Site Generation
  images: {
    unoptimized: true, // Required for static export
  },
  trailingSlash: true, // Better for static hosting
};

module.exports = nextConfig;

import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');

  return {
    plugins: [
      react(),
    ],
    server: {
      host: '0.0.0.0',
      port: 3000,
      allowedHosts: [
        env.WORKSPACE_DEV_DOMAIN
      ],
    },
  }
})
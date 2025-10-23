import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
	const vitePort = 3000;
	const vitePublicHost = `${env.WORKSPACE_ID}-${vitePort}.2.codesphere.com`; // Öffentliche URL für Vite
  const rustServerInternalHost = `http://ws-server-${env.WORKSPACE_ID}-server.workspaces:3000`; // Rust Server (HTTP)

  return {
    plugins: [
      react(),
    ],
    server: {
      host: '0.0.0.0',
      port: vitePort,
      allowedHosts: [
        env.WORKSPACE_DEV_DOMAIN
      ],
			proxy: {
        '/api': {
          target: rustServerInternalHost,
          changeOrigin: true,
          ws: true, 
          configure: (proxy, options) => { // Logging
            proxy.on('proxyReqWs', (proxyReq, req, socket, options, head) => {
              console.log(`[vite proxy] WS Request: ${req.url} -> ${options.target}${proxyReq.path}`);
            });
            proxy.on('error', (err, req, res) => { console.error('[vite proxy] Error: ', err); });
          }
        },
      },
    },
  }
})
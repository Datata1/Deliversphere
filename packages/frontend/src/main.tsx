import React from 'react'
import ReactDOM from 'react-dom/client'
import { RouterProvider } from '@tanstack/react-router'
import { StoreProvider } from './stores/StoreProvider' 
import { router } from './router' 
import './index.css' 

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <StoreProvider>
      <RouterProvider router={router} />
    </StoreProvider>
  </React.StrictMode>,
)
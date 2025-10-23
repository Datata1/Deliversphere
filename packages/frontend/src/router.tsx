import { RootRoute, Route, Router, Outlet } from '@tanstack/react-router';
import App from './App'; // Your main App component
import Dashboard from './pages/Dashboard'; // Example page component
import AgentDetails from './pages/AgentDetails'; // Example page component

// Create a root route (usually corresponds to your App layout)
const rootRoute = new RootRoute({
  component: () => (
    <>
      <h1>Deliversphere CI/CD</h1> {/* Example Layout Header */}
      <hr />
      <Outlet /> {/* Child routes render here */}
    </>
  ),
});

// Define specific routes as children of the root route
const indexRoute = new Route({
  getParentRoute: () => rootRoute,
  path: '/',
  component: Dashboard, // Component for the homepage
});

const agentRoute = new Route({
  getParentRoute: () => rootRoute,
  path: '/agents/$agentId', // Dynamic route parameter
  component: AgentDetails,
});

// Combine routes into a route tree
const routeTree = rootRoute.addChildren([indexRoute, agentRoute]);

// Create the router instance
export const router = new Router({ routeTree });

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
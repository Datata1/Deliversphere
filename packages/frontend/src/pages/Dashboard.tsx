import React, { useEffect, useState, useRef } from 'react'; // <-- useRef added
import { observer } from 'mobx-react-lite';
import { useStore } from '../stores/StoreProvider';
import { Link } from '@tanstack/react-router';
import { runInAction } from 'mobx';

// Interfaces (no change)
interface Agent {
  id: string;
  status: 'online' | 'offline' | 'busy' | 'unknown';
  hostname?: string;
  last_heartbeat?: number;
}
interface WsInitialState {
    type: 'InitialState';
    agents: Agent[];
}
interface WsAgentUpdate {
    type: 'AgentUpdate';
    agent: Agent;
}
interface WsStatsUpdate {
    type: 'StatsUpdate';
    online: number;
    offline: number;
}
type WsServerMessage = WsInitialState | WsAgentUpdate | WsStatsUpdate;


export const Dashboard = observer(() => {
  const { agentStore } = useStore();
  // --- KORREKTUR 1: Fehlende Hooks wieder hinzufügen ---
  const [isConnected, setIsConnected] = useState(false);
  const ws = useRef<WebSocket | null>(null);
  // ---------------------------------------------------

  // --- KORREKTUR 2: WebSocket-Logik in useEffect kapseln ---
  useEffect(() => {
    // Definiere die Funktion innerhalb des Effects
    function connectWebSocket() {
      console.log("Connecting to WebSocket...");
      const wsHost = window.location.host;
      const wsUrl = `wss://71635-3000.2.codesphere.com/server/api/ws`;

      ws.current = new WebSocket(wsUrl);

      ws.current.onopen = () => {
        console.log("WebSocket connected!");
        setIsConnected(true);
      };

      ws.current.onclose = (event) => {
        console.log("WebSocket disconnected:", event.code, event.reason);
        setIsConnected(false);
        ws.current = null;
        // Versuche nach 5 Sekunden erneut zu verbinden
        setTimeout(connectWebSocket, 5000);
      };

      ws.current.onerror = (error) => {
        console.error("WebSocket Error:", error);
      };

      ws.current.onmessage = (event) => {
        console.log('WebSocket message received:', event.data);
        try {
          const message: WsServerMessage = JSON.parse(event.data);
          runInAction(() => {
            switch (message.type) {
              case 'InitialState':
                console.log("Processing Initial State:", message.agents.length, "agents");
                agentStore.agents.clear();
                message.agents.forEach(agent => {
                    const validStatus = ['online', 'offline', 'busy'].includes(agent.status)
                        ? agent.status as Agent['status'] : 'unknown';
                    agentStore.agents.set(agent.id, { ...agent, status: validStatus });
                });
                break;
              case 'AgentUpdate':
                console.log(`Processing Agent Update for ${message.agent.id}`);
                agentStore.updateAgent(message.agent);
                break;
              case 'StatsUpdate':
                console.log(`Processing Stats Update: Online=${message.online}, Offline=${message.offline}`);
                // TODO: Handle this if you have a place to store these numbers
                break;
              default:
                console.warn("Received unknown WebSocket message type");
            }
          });
        } catch (error) {
          console.error("Failed to parse WebSocket message:", error);
        }
      };
    }

    // Starte die Verbindung, wenn die Komponente geladen wird
    connectWebSocket();

    // Cleanup-Funktion: Wird ausgeführt, wenn die Komponente unmountet wird
    return () => {
      console.log("Closing WebSocket connection...");
      ws.current?.close();
    };
  }, [agentStore]); // Abhängigkeit vom Store
  // --- ENDE KORREKTUR 2 ---

  return (
    <div>
      <h2>Dashboard</h2>
      <p>WebSocket Status: {isConnected ? 'Connected' : 'Disconnected'}</p>
      <p>Online Agents: {agentStore.onlineAgents.length}</p>
      <p>Offline Agents: {agentStore.offlineAgents.length}</p>

      {agentStore.isLoading && <p>Loading initial agent list...</p>}
      {agentStore.error && <p style={{ color: 'red' }}>Error loading agents: {agentStore.error}</p>}

      <h3>Online Agents</h3>
      {agentStore.onlineAgents.length === 0 && !agentStore.isLoading && <p>No agents currently online.</p>}
      <ul>
        {agentStore.onlineAgents.map((agent) => (
          <li key={agent.id}>
            <Link to="/agents/$agentId" params={{ agentId: agent.id }}>
              {agent.id} ({agent.hostname || '...'})
            </Link>
          </li>
        ))}
      </ul>

      <h3>Offline Agents</h3>
      {agentStore.offlineAgents.length === 0 && !agentStore.isLoading && <p>No agents currently offline.</p>}
      <ul>
        {agentStore.offlineAgents.map((agent) => (
          <li key={agent.id} style={{ color: 'gray' }}>
            {agent.id} ({agent.hostname || '...'}) - Offline
          </li>
        ))}
      </ul>
    </div>
  );
});

export default Dashboard;

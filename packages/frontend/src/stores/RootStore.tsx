import { makeAutoObservable } from "mobx";

interface Agent {
  id: string;
  status: 'online' | 'offline' | 'busy' | 'unknown'; 
  hostname?: string; 
  last_heartbeat?: number; 
}

class AgentStore {
  // Store agents as an array of objects
  agents = new Map<string, Agent>(); // Use a Map for easier lookup by ID
  isLoading = false;
  error: string | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  // Action to update or add an agent based on SSE data
  updateAgent(agentData: { id: string; status: string }) {
    const existingAgent = this.agents.get(agentData.id);
    const validStatus = ['online', 'offline', 'busy'].includes(agentData.status)
        ? agentData.status as Agent['status']
        : 'unknown';

    if (existingAgent) {
      // Update existing agent status
      existingAgent.status = validStatus;
    } else {
      // Add new agent
      this.agents.set(agentData.id, {
        id: agentData.id,
        status: validStatus,
      });
    }
  }

	async fetchInitialAgents() {
    if (this.isLoading) return;

    this.isLoading = true;
    this.error = null;
    try {
      const response = await fetch('/api/agents');
      if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
      const data: Agent[] = await response.json();
      runInAction(() => {
        this.agents.clear(); 
        data.forEach(agent => {
             const validStatus = ['online', 'offline', 'busy'].includes(agent.status)
                ? agent.status as Agent['status']
                : 'unknown';
             this.agents.set(agent.id, { ...agent, status: validStatus });
        });
        this.isLoading = false;
        console.log("Initial agent state loaded:", this.agents.size);
      });
    } catch (e) {
      console.error("Failed to fetch initial agents:", e);
      runInAction(() => {
        this.error = e instanceof Error ? e.message : String(e);
        this.isLoading = false;
      });
    }
  }

  // Action to remove an agent (useful if server sends 'removed' event later)
  removeAgent(agentId: string) {
    this.agents.delete(agentId);
  }


  // Computed values to get lists of agents by status
  get onlineAgents(): Agent[] {
    return Array.from(this.agents.values()).filter(agent => agent.status === 'online');
  }

  get offlineAgents(): Agent[] {
     return Array.from(this.agents.values()).filter(agent => agent.status === 'offline');
  }

   get agentCount() {
     return this.agents.size;
   }

  // Optional: Keep fetchAgents if you want an initial load via REST
  async fetchAgents() {
    this.isLoading = true;
    this.error = null;
    try {
      // Assumes API returns Agent[]
      const response = await fetch('/api/agents'); // Example REST endpoint
      if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
      const data: Agent[] = await response.json();
      runInAction(() => {
        this.agents.clear(); // Clear existing before loading all
        data.forEach(agent => this.agents.set(agent.id, agent));
        this.isLoading = false;
      });
    } catch (e) {
      console.error("Failed to fetch agents:", e);
      runInAction(() => {
        this.error = e instanceof Error ? e.message : String(e);
        this.isLoading = false;
      });
    }
  }
}

// RootStore remains the same
export class RootStore {
  agentStore: AgentStore;
  constructor() {
    this.agentStore = new AgentStore();
    makeAutoObservable(this);
  }
}

export const rootStore = new RootStore();
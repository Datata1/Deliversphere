import React from 'react';
import { useParams } from '@tanstack/react-router';

const AgentDetails = () => {
  // Access route params using useParams hook
  const { agentId } = useParams({ from: '/agents/$agentId' });

  return (
    <div>
      <h2>Agent Details</h2>
      <p>Viewing details for Agent ID: <strong>{agentId}</strong></p>
    </div>
  );
};

export default AgentDetails;
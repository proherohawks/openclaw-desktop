/**
 * All backend communication goes through this file.
 * Components NEVER call invoke() directly.
 */
import { invoke } from "@tauri-apps/api/core";
import type { Agent, AgentReply, ConnectionStatus, ConnectResult } from "./types";

export async function connect(endpoint: string, token: string): Promise<ConnectResult> {
  return invoke<ConnectResult>("connect", { endpoint, token });
}

export async function listAgents(): Promise<Agent[]> {
  return invoke<Agent[]>("list_agents");
}

export async function sendMessage(agentId: string, message: string): Promise<AgentReply> {
  return invoke<AgentReply>("send_message", { agentId, message });
}

export async function getStatus(): Promise<ConnectionStatus> {
  return invoke<ConnectionStatus>("get_status");
}

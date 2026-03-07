import { useStore } from "../lib/store";
import type { Agent } from "../lib/types";

function AgentRow({ agent }: { agent: Agent }) {
  const activeAgentId = useStore((s) => s.activeAgentId);
  const setActiveAgent = useStore((s) => s.setActiveAgent);
  const messageCount = useStore(
    (s) => (s.messages[agent.id] ?? []).length,
  );
  const isActive = agent.id === activeAgentId;

  const emoji = agent.identity?.emoji ?? "🤖";
  const displayName = agent.identity?.name ?? agent.name ?? agent.id;

  return (
    <button
      onClick={() => setActiveAgent(agent.id)}
      className={`flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left transition-all duration-150 ${
        isActive
          ? "border border-claw-border-hover bg-claw-card"
          : "border border-transparent hover:bg-claw-card/50"
      }`}
      style={{ marginBottom: 3 }}
    >
      <span className="text-base">{emoji}</span>
      <div className="flex-1 min-w-0">
        <div className={`truncate text-xs ${isActive ? "font-semibold text-claw-text-bright" : "text-claw-text-muted"}`}>
          {displayName}
        </div>
        <div className="flex items-center gap-1 text-[10px] text-claw-green">
          <span className="inline-block h-[5px] w-[5px] rounded-full bg-claw-green" />
          online
        </div>
      </div>
      {messageCount > 0 && (
        <span className="rounded bg-claw-border px-1.5 py-px text-[9px] text-claw-subtle">
          {messageCount}
        </span>
      )}
    </button>
  );
}

export default function Sidebar() {
  const agents = useStore((s) => s.agents);
  const disconnect = useStore((s) => s.disconnect);

  return (
    <div className="flex w-[220px] flex-shrink-0 flex-col border-r border-claw-border bg-claw-panel">
      {/* Logo */}
      <div className="border-b border-claw-border px-4 pb-3.5 pt-5">
        <div className="flex items-center gap-2">
          <span className="text-xl">🦞</span>
          <div>
            <div className="text-[13px] font-bold tracking-[0.08em] text-claw-amber">OPENCLAW</div>
            <div className="text-[10px] tracking-[0.1em] text-claw-subtle">AGENT CONSOLE</div>
          </div>
        </div>
      </div>

      {/* Agent list */}
      <div className="flex-1 overflow-y-auto px-2 py-3">
        <div className="px-2 pb-2 text-[9px] tracking-[0.15em] text-claw-muted">AGENTS</div>
        {agents.length === 0 ? (
          <p className="px-2 text-[11px] text-claw-subtle">
            No agents found.
          </p>
        ) : (
          agents.map((agent) => (
            <AgentRow key={agent.id} agent={agent} />
          ))
        )}
      </div>

      {/* Bottom controls */}
      <div className="border-t border-claw-border p-2">
        <button
          onClick={disconnect}
          className="flex w-full items-center gap-2 rounded-md border border-transparent px-2.5 py-2 text-[11px] text-claw-subtle transition-all duration-150 hover:bg-claw-border hover:text-claw-text-muted"
        >
          ⚙ Disconnect
        </button>
      </div>
    </div>
  );
}

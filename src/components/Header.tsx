import { useCallback } from "react";
import { useStore } from "../lib/store";

export default function Header() {
  const activeAgentId = useStore((s) => s.activeAgentId);
  const agents = useStore((s) => s.agents);
  const endpoint = useStore((s) => s.endpoint);
  const clearMessages = useStore((s) => s.clearMessages);

  const agent = agents.find((a) => a.id === activeAgentId);

  const handleClear = useCallback(() => {
    if (!activeAgentId) return;
    if (window.confirm("Clear message history for this agent?")) {
      clearMessages(activeAgentId);
    }
  }, [activeAgentId, clearMessages]);

  if (!agent) return null;

  const emoji = agent.identity?.emoji ?? "🤖";
  const displayName = agent.identity?.name ?? agent.name ?? agent.id;

  return (
    <div className="flex items-center gap-3 border-b border-claw-border bg-claw-panel px-5 py-3.5">
      <span className="text-lg">{emoji}</span>
      <div className="flex-1">
        <div className="text-[13px] font-semibold text-claw-text-bright">{displayName} Agent</div>
        <div className="text-[10px] text-claw-muted">{endpoint}</div>
      </div>
      <button
        onClick={handleClear}
        className="rounded border border-claw-border-hover px-2.5 py-1 text-[10px] tracking-[0.05em] text-claw-subtle transition-colors hover:border-claw-border-focus hover:text-claw-text-muted"
      >
        CLEAR
      </button>
    </div>
  );
}

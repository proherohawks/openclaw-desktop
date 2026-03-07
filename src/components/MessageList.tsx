import { useEffect, useRef } from "react";
import { useStore } from "../lib/store";
import MessageBubble from "./MessageBubble";

export default function MessageList() {
  const bottomRef = useRef<HTMLDivElement>(null);
  const activeAgentId = useStore((s) => s.activeAgentId);
  const agents = useStore((s) => s.agents);
  const allMessages = useStore((s) => s.messages);
  const allSending = useStore((s) => s.sending);

  const agent = agents.find((a) => a.id === activeAgentId);
  const messages = activeAgentId ? allMessages[activeAgentId] ?? [] : [];
  const isSending = activeAgentId ? allSending[activeAgentId] ?? false : false;

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages.length, isSending]);

  if (!activeAgentId) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center text-claw-dim">
        <div className="text-4xl mb-3">🦞</div>
        <div className="text-[13px] text-claw-muted">Select an agent to start chatting</div>
      </div>
    );
  }

  const agentEmoji = agent?.identity?.emoji ?? "🤖";
  const agentName = agent?.identity?.name ?? agent?.name ?? agent?.id ?? "agent";

  return (
    <div className="flex flex-1 flex-col gap-3.5 overflow-y-auto p-5">
      {messages.length === 0 && !isSending && (
        <div className="m-auto text-center">
          <div className="mb-3 text-4xl">🦞</div>
          <div className="text-[13px] text-claw-muted">No messages yet</div>
          <div className="mt-1 text-[11px] text-claw-dim">Send something to the {agentName} agent</div>
        </div>
      )}

      {messages.map((msg) => (
        <MessageBubble key={msg.id} message={msg} agentEmoji={agentEmoji} />
      ))}

      {isSending && (
        <div className="flex items-end gap-2.5">
          <div className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-md border border-claw-border-hover bg-claw-card text-sm">
            {agentEmoji}
          </div>
          <div className="flex items-center gap-[5px] rounded-[10px_10px_10px_2px] border border-claw-border bg-claw-surface px-4 py-3">
            {[0, 1, 2].map((i) => (
              <span
                key={i}
                className="inline-block h-[5px] w-[5px] rounded-full bg-claw-amber animate-dot-pulse"
                style={{ animationDelay: `${i * 0.2}s` }}
              />
            ))}
          </div>
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
}

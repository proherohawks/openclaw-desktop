import { useState, useRef, useCallback, type KeyboardEvent, type ChangeEvent } from "react";
import { useStore } from "../lib/store";

export default function InputBar() {
  const [input, setInput] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const activeAgentId = useStore((s) => s.activeAgentId);
  const agents = useStore((s) => s.agents);
  const sending = useStore((s) => s.sending);
  const sendMessage = useStore((s) => s.sendMessage);

  const agent = agents.find((a) => a.id === activeAgentId);
  const isSending = activeAgentId ? sending[activeAgentId] ?? false : false;
  const canSend = input.trim().length > 0 && !isSending;

  const handleSend = useCallback(async () => {
    const text = input.trim();
    if (!text || isSending) return;
    setInput("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
    await sendMessage(text);
    setTimeout(() => textareaRef.current?.focus(), 50);
  }, [input, isSending, sendMessage]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
      if (e.key === "Escape") {
        textareaRef.current?.blur();
      }
    },
    [handleSend],
  );

  const handleChange = useCallback((e: ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    const el = e.target;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 120)}px`;
  }, []);

  const agentName = agent?.identity?.name ?? agent?.name ?? agent?.id ?? "agent";

  return (
    <div className="flex items-end gap-2.5 border-t border-claw-border bg-claw-panel px-5 py-3.5">
      <textarea
        ref={textareaRef}
        value={input}
        rows={1}
        className="flex-1 resize-none rounded-lg border border-claw-border-hover bg-claw-card px-3.5 py-2.5 text-[13px] leading-relaxed text-claw-text-bright outline-none transition-colors focus:border-claw-border-focus"
        placeholder={`Message ${agentName} agent... (Enter to send)`}
        onKeyDown={handleKeyDown}
        onChange={handleChange}
        disabled={isSending}
        style={{ minHeight: 42, maxHeight: 120 }}
      />
      <button
        onClick={handleSend}
        disabled={!canSend}
        className={`flex h-[42px] w-[42px] flex-shrink-0 items-center justify-center rounded-lg border text-base transition-all duration-150 ${
          canSend
            ? "border-claw-amber bg-claw-amber cursor-pointer"
            : "border-claw-border-hover bg-claw-card cursor-not-allowed"
        }`}
      >
        <span className={canSend ? "text-claw-bg" : "text-claw-muted"}>↑</span>
      </button>
    </div>
  );
}

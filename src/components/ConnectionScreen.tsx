import { useCallback, useEffect, useRef, type KeyboardEvent, type ChangeEvent } from "react";
import { useStore } from "../lib/store";

export default function ConnectionScreen() {
  const endpoint = useStore((s) => s.endpoint);
  const token = useStore((s) => s.token);
  const connectionState = useStore((s) => s.connectionState);
  const connectionError = useStore((s) => s.connectionError);
  const setEndpoint = useStore((s) => s.setEndpoint);
  const setToken = useStore((s) => s.setToken);
  const connect = useStore((s) => s.connect);

  const isConnecting = connectionState === "connecting";
  const autoConnectAttempted = useRef(false);

  // Auto-connect if we have saved credentials
  useEffect(() => {
    if (
      !autoConnectAttempted.current &&
      connectionState === "disconnected" &&
      endpoint.trim() &&
      token.trim()
    ) {
      autoConnectAttempted.current = true;
      connect(endpoint.trim(), token.trim());
    }
  }, [connectionState, endpoint, token, connect]);

  const handleConnect = useCallback(() => {
    if (!endpoint.trim() || !token.trim() || isConnecting) return;
    connect(endpoint.trim(), token.trim());
  }, [endpoint, token, isConnecting, connect]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") {
        handleConnect();
      }
    },
    [handleConnect],
  );

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-8 bg-claw-bg font-mono text-claw-text">
      {/* Logo */}
      <div className="text-center">
        <div className="mb-4 text-5xl">🦞</div>
        <div className="text-sm font-bold tracking-[0.08em] text-claw-amber">OPENCLAW</div>
        <div className="mt-1 text-[10px] tracking-[0.1em] text-claw-subtle">AGENT CONSOLE</div>
      </div>

      {/* Form */}
      <div className="flex w-80 flex-col gap-4">
        <div>
          <label className="mb-1.5 block text-[11px] text-claw-subtle">WebSocket URL</label>
          <input
            type="text"
            value={endpoint}
            onChange={(e: ChangeEvent<HTMLInputElement>) => setEndpoint(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="ws://127.0.0.1:18789"
            disabled={isConnecting}
            className="w-full rounded-lg border border-claw-border-hover bg-claw-card px-4 py-2.5 text-[13px] text-claw-text-bright outline-none transition-colors focus:border-claw-border-focus disabled:opacity-50"
          />
        </div>
        <div>
          <label className="mb-1.5 block text-[11px] text-claw-subtle">Gateway Token</label>
          <input
            type="password"
            value={token}
            onChange={(e: ChangeEvent<HTMLInputElement>) => setToken(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="your gateway auth token"
            disabled={isConnecting}
            className="w-full rounded-lg border border-claw-border-hover bg-claw-card px-4 py-2.5 text-[13px] text-claw-text-bright outline-none transition-colors focus:border-claw-border-focus disabled:opacity-50"
          />
        </div>
        <button
          onClick={handleConnect}
          disabled={isConnecting || !endpoint.trim() || !token.trim()}
          className="rounded-lg bg-claw-amber py-2.5 text-[13px] font-semibold text-claw-bg transition-colors hover:bg-claw-amber-light disabled:cursor-not-allowed disabled:opacity-40"
        >
          {isConnecting ? "Connecting..." : "Connect"}
        </button>
      </div>

      {connectionState === "error" && connectionError && (
        <div className="max-w-sm rounded-lg border border-claw-error-border bg-claw-error-bg px-4 py-3 text-center text-[13px] text-red-400">
          {connectionError}
        </div>
      )}
    </div>
  );
}

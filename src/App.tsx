import { useEffect } from "react";
import { useStore } from "./lib/store";
import ConnectionScreen from "./components/ConnectionScreen";
import Sidebar from "./components/Sidebar";
import Header from "./components/Header";
import MessageList from "./components/MessageList";
import InputBar from "./components/InputBar";

export default function App() {
  const connectionState = useStore((s) => s.connectionState);
  const activeAgentId = useStore((s) => s.activeAgentId);
  const clearMessages = useStore((s) => s.clearMessages);

  useEffect(() => {
    function handleGlobalKey(e: KeyboardEvent) {
      if (e.ctrlKey && e.key === "l") {
        e.preventDefault();
        if (activeAgentId && window.confirm("Clear message history for this agent?")) {
          clearMessages(activeAgentId);
        }
      }
    }
    window.addEventListener("keydown", handleGlobalKey);
    return () => window.removeEventListener("keydown", handleGlobalKey);
  }, [activeAgentId, clearMessages]);

  if (connectionState !== "connected") {
    return <ConnectionScreen />;
  }

  return (
    <div className="flex h-screen overflow-hidden bg-claw-bg font-mono text-claw-text">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        <MessageList />
        <InputBar />
      </div>
    </div>
  );
}

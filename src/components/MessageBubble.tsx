import type { Message } from "../lib/types";
import LinkifiedText from "./LinkifiedText";
import CopyButton from "./CopyButton";
import MarkdownContent from "./MarkdownContent";

interface Props {
  message: Message;
  agentEmoji: string;
}

export default function MessageBubble({ message, agentEmoji }: Props) {
  const isUser = message.role === "user";
  const isError = message.role === "error";
  const ts = new Date(message.ts).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });

  return (
    <div className={`flex items-end gap-2.5 ${isUser ? "flex-row-reverse" : "flex-row"}`}>
      {!isUser && (
        <div
          className={`flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-md border text-sm ${
            isError
              ? "border-claw-error-avatar-border bg-claw-error-avatar-bg"
              : "border-claw-border-hover bg-claw-card"
          }`}
        >
          {isError ? "⚠" : agentEmoji}
        </div>
      )}

      <div className="group relative max-w-[68%]">
        <CopyButton
          text={message.text || ""}
          className={`absolute -top-2 z-10 opacity-0 transition-opacity group-hover:opacity-100 ${
            isUser ? "-left-2" : "-right-2"
          }`}
        />
        <div
          className={`border px-3.5 py-2.5 ${
            isUser
              ? "rounded-[10px_10px_2px_10px] border-claw-user-border bg-claw-user-bg"
              : isError
                ? "rounded-[10px_10px_10px_2px] border-claw-error-border bg-claw-error-bg"
                : "rounded-[10px_10px_10px_2px] border-claw-border bg-claw-surface"
          }`}
        >
          <div
            className={`break-words text-[13px] leading-relaxed ${
              isUser
                ? "whitespace-pre-wrap text-claw-amber-light"
                : isError
                  ? "whitespace-pre-wrap text-red-400"
                  : "text-[#d4d4d8]"
            }`}
          >
            {isUser || isError ? (
              <LinkifiedText text={message.text || "(empty reply)"} />
            ) : (
              <MarkdownContent content={message.text || "(empty reply)"} />
            )}
          </div>
          <div className="mt-1.5 text-right text-[9px] text-claw-dim">{ts}</div>
        </div>
      </div>
    </div>
  );
}

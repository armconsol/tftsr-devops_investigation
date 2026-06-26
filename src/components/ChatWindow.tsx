import React, { useState, useRef, useEffect } from "react";
import { Send, Bot, User, Paperclip, X } from "lucide-react";
import type { TriageMessage } from "@/lib/tauriCommands";

interface ChatWindowProps {
  messages: TriageMessage[];
  onSend: (message: string) => Promise<void>;
  isLoading?: boolean;
  placeholder?: string;
  pendingFiles?: { name: string }[];
  onAttach?: () => void;
  onRemoveFile?: (index: number) => void;
}

export function ChatWindow({ messages, onSend, isLoading, placeholder, pendingFiles, onAttach, onRemoveFile }: ChatWindowProps) {
  const [input, setInput] = useState("");
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || isLoading) return;
    const msg = input;
    setInput("");
    await onSend(msg);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex gap-3 ${msg.role === "user" ? "justify-end" : "justify-start"}`}
          >
            {msg.role === "assistant" && (
              <div className="w-8 h-8 rounded-full bg-primary flex items-center justify-center shrink-0">
                <Bot className="w-4 h-4 text-primary-foreground" />
              </div>
            )}
            <div
              className={`max-w-[75%] rounded-lg px-4 py-2 text-sm ${
                msg.role === "user"
                  ? "bg-primary text-primary-foreground"
                  : "bg-muted text-foreground"
              }`}
            >
              <p className="whitespace-pre-wrap">{msg.content}</p>
            </div>
            {msg.role === "user" && (
              <div className="w-8 h-8 rounded-full bg-secondary flex items-center justify-center shrink-0">
                <User className="w-4 h-4 text-secondary-foreground" />
              </div>
            )}
          </div>
        ))}
        {isLoading && (
          <div className="flex gap-3 justify-start">
            <div className="w-8 h-8 rounded-full bg-primary flex items-center justify-center">
              <Bot className="w-4 h-4 text-primary-foreground" />
            </div>
            <div className="bg-muted rounded-lg px-4 py-3">
              <div className="flex space-x-1">
                <div className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce" />
                <div className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce [animation-delay:0.1s]" />
                <div className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce [animation-delay:0.2s]" />
              </div>
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>
      <div className="border-t p-4 space-y-2">
        {pendingFiles && pendingFiles.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {pendingFiles.map((f, i) => (
              <span
                key={i}
                className="inline-flex items-center gap-1 rounded-md bg-secondary px-2 py-1 text-xs text-secondary-foreground"
              >
                <Paperclip className="w-3 h-3" />
                {f.name}
                {onRemoveFile && (
                  <button onClick={() => onRemoveFile(i)} className="ml-1 hover:text-destructive">
                    <X className="w-3 h-3" />
                  </button>
                )}
              </span>
            ))}
          </div>
        )}
        <div className="flex gap-2">
          {onAttach && (
            <button
              onClick={onAttach}
              disabled={isLoading}
              title="Attach log file or screenshot"
              className="px-3 py-2 rounded-md border border-input bg-background hover:bg-accent disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Paperclip className="w-4 h-4 text-muted-foreground" />
            </button>
          )}
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={placeholder ?? "Type your response... (Enter to send, Shift+Enter for new line)"}
            rows={2}
            className="flex-1 resize-none rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            disabled={isLoading}
          />
          <button
            onClick={handleSend}
            disabled={(!input.trim() && (!pendingFiles || pendingFiles.length === 0)) || isLoading}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}

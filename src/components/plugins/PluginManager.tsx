import { useState } from "react";
import { Button } from "@/components/ui/button";
import ClaudePluginTab from "./ClaudePluginTab";
import OpencodePluginTab from "./OpencodePluginTab";
import OpenclawPluginTab from "./OpenclawPluginTab";
import CodexPluginTab from "./CodexPluginTab";
import HermesPluginTab from "./HermesPluginTab";
import GeminiPluginTab from "./GeminiPluginTab";

type PluginTab =
  | "claude"
  | "opencode"
  | "openclaw"
  | "codex"
  | "hermes"
  | "gemini";

export default function PluginManager() {
  const [activeTab, setActiveTab] = useState<PluginTab>("claude");

  return (
    <div className="px-6 flex flex-col flex-1 min-h-0 overflow-hidden">
      <div className="flex items-center gap-1 mb-4 mt-2">
        <Button
          variant={activeTab === "claude" ? "default" : "ghost"}
          size="sm"
          onClick={() => setActiveTab("claude")}
        >
          Claude
        </Button>
        <Button
          variant={activeTab === "codex" ? "default" : "ghost"}
          size="sm"
          onClick={() => setActiveTab("codex")}
        >
          Codex
        </Button>
        <Button
          variant={activeTab === "gemini" ? "default" : "ghost"}
          size="sm"
          onClick={() => setActiveTab("gemini")}
        >
          Gemini
        </Button>
        <Button
          variant={activeTab === "opencode" ? "default" : "ghost"}
          size="sm"
          onClick={() => setActiveTab("opencode")}
        >
          OpenCode
        </Button>
        <Button
          variant={activeTab === "openclaw" ? "default" : "ghost"}
          size="sm"
          onClick={() => setActiveTab("openclaw")}
        >
          OpenClaw
        </Button>
        <Button
          variant={activeTab === "hermes" ? "default" : "ghost"}
          size="sm"
          onClick={() => setActiveTab("hermes")}
        >
          Hermes
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto overflow-x-hidden pb-24">
        {activeTab === "claude" && <ClaudePluginTab />}
        {activeTab === "codex" && <CodexPluginTab />}
        {activeTab === "gemini" && <GeminiPluginTab />}
        {activeTab === "opencode" && <OpencodePluginTab />}
        {activeTab === "openclaw" && <OpenclawPluginTab />}
        {activeTab === "hermes" && <HermesPluginTab />}
      </div>
    </div>
  );
}

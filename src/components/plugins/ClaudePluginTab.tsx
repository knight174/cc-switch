import { useMemo } from "react";
import {
  useClaudePlugins,
  useClaudeInstalledPlugins,
  useApplyClaudePluginSelection,
  useImportClaudePlugins,
} from "@/hooks/useClaudePlugins";
import { useCheckboxPluginTab } from "@/hooks/useCheckboxPluginTab";
import CheckboxPluginTab from "./CheckboxPluginTab";
import type { ClaudeInstalledPlugin } from "@/lib/api/claudePlugins";

export default function ClaudePluginTab() {
  const { data: globalPlugins, isLoading: globalLoading } = useClaudePlugins();
  const { data: installedPlugins, isLoading: installedLoading } =
    useClaudeInstalledPlugins();
  const applyMutation = useApplyClaudePluginSelection();
  const importMutation = useImportClaudePlugins();

  const initialEnabledIds = useMemo(() => {
    if (!globalPlugins) return undefined;
    return new Set(
      Object.entries(globalPlugins)
        .filter(([, enabled]) => enabled)
        .map(([id]) => id),
    );
  }, [globalPlugins]);

  const { selectedIds, hasChanges, isApplying, isImporting, handleToggle, handleApply, handleImport } =
    useCheckboxPluginTab({
      installedPlugins,
      isLoading: globalLoading || installedLoading,
      initialEnabledIds,
      applyMutation,
      importMutation,
    });

  return (
    <CheckboxPluginTab<ClaudeInstalledPlugin>
      providerKey="claude"
      installedPlugins={installedPlugins}
      isLoading={globalLoading || installedLoading}
      selectedIds={selectedIds}
      hasChanges={hasChanges}
      isApplying={isApplying}
      isImporting={isImporting}
      onToggle={handleToggle}
      onApply={handleApply}
      onImport={handleImport}
      renderPluginMeta={(plugin) => `v${plugin.version} · ${plugin.scope}`}
    />
  );
}

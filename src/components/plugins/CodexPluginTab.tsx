import { useMemo } from "react";
import {
  useCodexInstalledPlugins,
  useApplyCodexPluginSelection,
  useImportCodexPlugins,
} from "@/hooks/useCodexPlugins";
import { useCheckboxPluginTab } from "@/hooks/useCheckboxPluginTab";
import CheckboxPluginTab from "./CheckboxPluginTab";
import type { CodexInstalledPlugin } from "@/lib/api/codexPlugins";

export default function CodexPluginTab() {
  const { data: installedPlugins, isLoading } = useCodexInstalledPlugins();
  const applyMutation = useApplyCodexPluginSelection();
  const importMutation = useImportCodexPlugins();

  const initialEnabledIds = useMemo(() => {
    if (!installedPlugins) return undefined;
    return new Set(installedPlugins.filter((p) => p.enabled).map((p) => p.id));
  }, [installedPlugins]);

  const {
    selectedIds,
    hasChanges,
    isApplying,
    isImporting,
    handleToggle,
    handleApply,
    handleImport,
  } = useCheckboxPluginTab({
    installedPlugins,
    isLoading,
    initialEnabledIds,
    applyMutation,
    importMutation,
  });

  return (
    <CheckboxPluginTab<CodexInstalledPlugin>
      providerKey="codex"
      installedPlugins={installedPlugins}
      isLoading={isLoading}
      selectedIds={selectedIds}
      hasChanges={hasChanges}
      isApplying={isApplying}
      isImporting={isImporting}
      onToggle={handleToggle}
      onApply={handleApply}
      onImport={handleImport}
    />
  );
}

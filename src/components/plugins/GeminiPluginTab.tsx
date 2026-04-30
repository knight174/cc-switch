import { useMemo } from "react";
import {
  useGeminiInstalledPlugins,
  useApplyGeminiPluginSelection,
  useImportGeminiPlugins,
} from "@/hooks/useGeminiPlugins";
import { useCheckboxPluginTab } from "@/hooks/useCheckboxPluginTab";
import CheckboxPluginTab from "./CheckboxPluginTab";
import type { GeminiInstalledPlugin } from "@/lib/api/geminiPlugins";

export default function GeminiPluginTab() {
  const { data: installedPlugins, isLoading } = useGeminiInstalledPlugins();
  const applyMutation = useApplyGeminiPluginSelection();
  const importMutation = useImportGeminiPlugins();

  const initialEnabledIds = useMemo(() => {
    if (!installedPlugins) return undefined;
    return new Set(
      installedPlugins.filter((p) => p.enabled).map((p) => p.id),
    );
  }, [installedPlugins]);

  const { selectedIds, hasChanges, isApplying, isImporting, handleToggle, handleApply, handleImport } =
    useCheckboxPluginTab({
      installedPlugins,
      isLoading,
      initialEnabledIds,
      applyMutation,
      importMutation,
    });

  return (
    <CheckboxPluginTab<GeminiInstalledPlugin>
      providerKey="gemini"
      installedPlugins={installedPlugins}
      isLoading={isLoading}
      selectedIds={selectedIds}
      hasChanges={hasChanges}
      isApplying={isApplying}
      isImporting={isImporting}
      onToggle={handleToggle}
      onApply={handleApply}
      onImport={handleImport}
      renderPluginMeta={(plugin) =>
        plugin.version ? `v${plugin.version}` : undefined
      }
    />
  );
}

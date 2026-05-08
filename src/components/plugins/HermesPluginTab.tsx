import { useMemo } from "react";
import {
  useHermesInstalledPlugins,
  useApplyHermesPluginSelection,
  useImportHermesPlugins,
} from "@/hooks/useHermesPlugins";
import { useCheckboxPluginTab } from "@/hooks/useCheckboxPluginTab";
import CheckboxPluginTab from "./CheckboxPluginTab";
import type { HermesInstalledPlugin } from "@/lib/api/hermesPlugins";

export default function HermesPluginTab() {
  const { data: installedPlugins, isLoading } = useHermesInstalledPlugins();
  const applyMutation = useApplyHermesPluginSelection();
  const importMutation = useImportHermesPlugins();

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
    <CheckboxPluginTab<HermesInstalledPlugin>
      providerKey="hermes"
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

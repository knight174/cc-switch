import { useMemo } from "react";
import {
  useOpenclawInstalledPlugins,
  useApplyOpenclawPluginSelection,
  useImportOpenclawPlugins,
} from "@/hooks/useOpenclawPlugins";
import { useCheckboxPluginTab } from "@/hooks/useCheckboxPluginTab";
import CheckboxPluginTab from "./CheckboxPluginTab";
import type { OpenClawInstalledPlugin } from "@/lib/api/openclawPlugins";

export default function OpenclawPluginTab() {
  const { data: installedPlugins, isLoading } = useOpenclawInstalledPlugins();
  const applyMutation = useApplyOpenclawPluginSelection();
  const importMutation = useImportOpenclawPlugins();

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
    <CheckboxPluginTab<OpenClawInstalledPlugin>
      providerKey="openclaw"
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

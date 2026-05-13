import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Loader2, Trash2, RefreshCw, ArrowUp } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  useClaudePlugins,
  useClaudeInstalledPlugins,
  useApplyClaudePluginSelection,
  useImportClaudePlugins,
} from "@/hooks/useClaudePlugins";
import {
  useUninstallClaudePlugin,
  useUpdateClaudePlugin,
  useClaudeMarketplaceList,
  useCheckClaudePluginUpdates,
} from "@/hooks/useClaudeMarketplace";
import { useCheckboxPluginTab } from "@/hooks/useCheckboxPluginTab";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import CheckboxPluginTab from "./CheckboxPluginTab";
import ClaudeDiscoverTab from "./ClaudeDiscoverTab";
import type { ClaudeInstalledPlugin } from "@/lib/api/claudePlugins";
import { toast } from "sonner";
import { extractErrorMessage } from "@/utils/errorUtils";

type ClaudeSubTab = "installed" | "discover";

export default function ClaudePluginTab() {
  const { t } = useTranslation();
  const [subTab, setSubTab] = useState<ClaudeSubTab>("installed");
  const [actionId, setActionId] = useState<string | null>(null);
  const [updatingAll, setUpdatingAll] = useState(false);
  const [uninstallTarget, setUninstallTarget] = useState<{
    id: string;
    name: string;
  } | null>(null);

  const { data: globalPlugins, isLoading: globalLoading } = useClaudePlugins();
  const { data: installedPlugins, isLoading: installedLoading } =
    useClaudeInstalledPlugins();
  const applyMutation = useApplyClaudePluginSelection();
  const importMutation = useImportClaudePlugins();
  const uninstallMutation = useUninstallClaudePlugin();
  const updateMutation = useUpdateClaudePlugin();
  const { data: marketplaceData } = useClaudeMarketplaceList();
  const {
    data: updateInfo,
    refetch: refetchUpdates,
    isFetching: isCheckingUpdates,
  } = useCheckClaudePluginUpdates();

  const descriptionMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const p of marketplaceData?.available ?? []) {
      if (p.description) map.set(p.pluginId, p.description);
    }
    return map;
  }, [marketplaceData?.available]);

  const updatableIds = useMemo(() => {
    return new Set(
      (updateInfo ?? []).filter((u) => u.has_update).map((u) => u.id),
    );
  }, [updateInfo]);

  const initialEnabledIds = useMemo(() => {
    if (!globalPlugins) return undefined;
    return new Set(
      Object.entries(globalPlugins)
        .filter(([, enabled]) => enabled)
        .map(([id]) => id),
    );
  }, [globalPlugins]);

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
    isLoading: globalLoading || installedLoading,
    initialEnabledIds,
    applyMutation,
    importMutation,
  });

  const doUninstall = async (pluginId: string, name: string) => {
    setActionId(pluginId);
    try {
      await uninstallMutation.mutateAsync(pluginId);
      toast.success(t("plugins.claude.uninstallSuccess", { name }));
    } catch (err) {
      toast.error(t("plugins.claude.uninstallFailed"), {
        description: extractErrorMessage(err),
      });
    } finally {
      setActionId(null);
    }
  };

  const handleUpdate = async (pluginId: string, name: string) => {
    setActionId(pluginId);
    try {
      await updateMutation.mutateAsync(pluginId);
      toast.success(t("plugins.claude.updateSuccess", { name }));
      await refetchUpdates();
    } catch (err) {
      toast.error(t("plugins.claude.updateFailed"), {
        description: extractErrorMessage(err),
      });
    } finally {
      setActionId(null);
    }
  };

  const handleCheckUpdates = async () => {
    try {
      const result = await refetchUpdates();
      const count = (result.data ?? []).filter((u) => u.has_update).length;
      if (count > 0) {
        toast.success(t("plugins.claude.updatesAvailable", { count }));
      } else {
        toast.success(t("plugins.claude.allUpToDate"));
      }
    } catch (err) {
      toast.error(t("plugins.claude.checkUpdatesFailed"), {
        description: extractErrorMessage(err),
      });
    }
  };

  const handleUpdateAll = async () => {
    const ids = Array.from(updatableIds);
    if (ids.length === 0) return;
    setUpdatingAll(true);
    let success = 0;
    let failed = 0;
    for (const id of ids) {
      try {
        await updateMutation.mutateAsync(id);
        success++;
      } catch {
        failed++;
      }
    }
    setUpdatingAll(false);
    await refetchUpdates();
    if (failed === 0) {
      toast.success(t("plugins.claude.updateAllSuccess", { count: success }));
    } else {
      toast.error(t("plugins.claude.updateAllPartial", { success, failed }));
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex gap-1">
          <Button
            size="sm"
            variant={subTab === "installed" ? "default" : "ghost"}
            onClick={() => setSubTab("installed")}
          >
            {t("plugins.claude.installed")}
          </Button>
          <Button
            size="sm"
            variant={subTab === "discover" ? "default" : "ghost"}
            onClick={() => setSubTab("discover")}
          >
            {t("plugins.claude.discover")}
          </Button>
        </div>

        {subTab === "installed" && (
          <div className="flex items-center gap-2">
            {updatableIds.size > 0 && (
              <Button
                size="sm"
                variant="default"
                onClick={handleUpdateAll}
                disabled={updatingAll}
              >
                {updatingAll ? (
                  <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                ) : (
                  <ArrowUp className="w-3 h-3 mr-1" />
                )}
                {t("plugins.claude.updateAll", { count: updatableIds.size })}
              </Button>
            )}
            <Button
              size="sm"
              variant="outline"
              onClick={handleCheckUpdates}
              disabled={isCheckingUpdates}
            >
              {isCheckingUpdates ? (
                <Loader2 className="w-3 h-3 mr-1 animate-spin" />
              ) : (
                <RefreshCw className="w-3 h-3 mr-1" />
              )}
              {t("plugins.claude.checkUpdates")}
            </Button>
          </div>
        )}
      </div>

      {subTab === "installed" && (
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
          renderPluginMeta={(plugin) => (
            <span className="flex items-center gap-2">
              <span>
                v{plugin.version} · {plugin.scope}
                {descriptionMap.has(plugin.id) && (
                  <span
                    className="ml-2 text-muted-foreground/70 truncate inline-block max-w-[200px] align-bottom"
                    title={descriptionMap.get(plugin.id)}
                  >
                    — {descriptionMap.get(plugin.id)}
                  </span>
                )}
              </span>
              {updatableIds.has(plugin.id) && (
                <Badge
                  variant="default"
                  className="bg-amber-500/90 hover:bg-amber-500 dark:bg-amber-600/90 dark:hover:bg-amber-600 text-white border-0 text-[10px] px-1.5 py-0 h-4"
                >
                  {t("plugins.claude.updateAvailable")}
                </Badge>
              )}
              <Button
                variant="ghost"
                size="sm"
                className="h-5 px-1.5 text-xs"
                disabled={actionId === plugin.id}
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  handleUpdate(plugin.id, plugin.id.split("@")[0]);
                }}
              >
                {actionId === plugin.id && updateMutation.isPending ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <RefreshCw className="w-3 h-3" />
                )}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-5 px-1.5 text-xs text-muted-foreground hover:text-destructive"
                disabled={actionId === plugin.id}
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  setUninstallTarget({
                    id: plugin.id,
                    name: plugin.id.split("@")[0],
                  });
                }}
              >
                {actionId === plugin.id && uninstallMutation.isPending ? (
                  <Loader2 className="w-3 h-3 animate-spin" />
                ) : (
                  <Trash2 className="w-3 h-3" />
                )}
              </Button>
            </span>
          )}
        />
      )}

      {subTab === "discover" && <ClaudeDiscoverTab />}

      <ConfirmDialog
        isOpen={!!uninstallTarget}
        title={t("plugins.claude.uninstall")}
        message={t("plugins.claude.uninstallConfirm", {
          name: uninstallTarget?.name ?? "",
        })}
        variant="destructive"
        onConfirm={() => {
          if (uninstallTarget) {
            doUninstall(uninstallTarget.id, uninstallTarget.name);
          }
          setUninstallTarget(null);
        }}
        onCancel={() => setUninstallTarget(null)}
      />
    </div>
  );
}

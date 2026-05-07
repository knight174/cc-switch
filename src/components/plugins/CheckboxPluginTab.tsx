import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Download, Check, Puzzle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";

interface PluginItem {
  id: string;
}

interface CheckboxPluginTabProps<T extends PluginItem> {
  providerKey: string;
  installedPlugins: T[] | undefined;
  isLoading: boolean;
  selectedIds: Set<string>;
  hasChanges: boolean;
  isApplying: boolean;
  isImporting: boolean;
  onToggle: (id: string, checked: boolean) => void;
  onApply: () => void;
  onImport: () => void;
  renderPluginMeta?: (plugin: T) => ReactNode;
}

export default function CheckboxPluginTab<T extends PluginItem>(
  props: CheckboxPluginTabProps<T>,
) {
  const { t } = useTranslation();
  const {
    providerKey,
    installedPlugins,
    isLoading,
    selectedIds,
    hasChanges,
    isApplying,
    isImporting,
    onToggle,
    onApply,
    onImport,
    renderPluginMeta,
  } = props;

  if (isLoading) {
    return (
      <div className="text-muted-foreground text-sm">{t("common.loading")}</div>
    );
  }

  if (!installedPlugins || installedPlugins.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="w-16 h-16 mx-auto mb-4 bg-muted rounded-full flex items-center justify-center">
          <Puzzle className="w-6 h-6 text-muted-foreground" />
        </div>
        <h3 className="text-lg font-medium text-foreground mb-2">
          {t(`plugins.${providerKey}.empty`)}
        </h3>
        <p className="text-muted-foreground text-sm mb-4">
          {t(`plugins.${providerKey}.emptyDescription`)}
        </p>
        <Button variant="outline" size="sm" onClick={onImport}>
          <Download className="w-4 h-4 mr-1" />
          {t("plugins.import")}
        </Button>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <span className="text-sm text-muted-foreground">
          {t(`plugins.${providerKey}.installedCount`, {
            count: installedPlugins.length,
          })}
        </span>
        <div className="flex items-center gap-2">
          <Button
            variant="default"
            size="sm"
            disabled={!hasChanges || isApplying}
            onClick={onApply}
          >
            <Check className="w-4 h-4 mr-1" />
            {isApplying ? t("common.applying") : t("common.apply")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={onImport}
            disabled={isImporting}
          >
            <Download className="w-4 h-4 mr-1" />
            {t("plugins.import")}
          </Button>
        </div>
      </div>

      <div className="rounded-xl border border-border-default overflow-hidden">
        {installedPlugins.map((plugin, index) => (
          <div
            key={plugin.id}
            className={`flex items-center justify-between px-4 py-3 ${
              index !== installedPlugins.length - 1
                ? "border-b border-border-default"
                : ""
            }`}
          >
            <div className="flex items-center gap-3 flex-1 min-w-0">
              <Checkbox
                id={`${providerKey}-plugin-${plugin.id}`}
                checked={selectedIds.has(plugin.id)}
                onCheckedChange={(checked) =>
                  onToggle(plugin.id, checked === true)
                }
              />
              <label
                htmlFor={`${providerKey}-plugin-${plugin.id}`}
                className="flex flex-col min-w-0 cursor-pointer"
              >
                <span className="text-sm font-medium truncate">
                  {plugin.id}
                </span>
                {renderPluginMeta && (
                  <span className="text-xs text-muted-foreground">
                    {renderPluginMeta(plugin)}
                  </span>
                )}
              </label>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

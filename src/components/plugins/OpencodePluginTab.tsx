import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Trash2, Download, ArrowUp, ArrowDown, Puzzle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  useOpencodePlugins,
  useAddOpencodePlugin,
  useRemoveOpencodePlugin,
  useReorderOpencodePlugins,
  useImportOpencodePlugins,
} from "@/hooks/useOpencodePlugins";
import { toast } from "sonner";
import { extractErrorMessage } from "@/utils/errorUtils";

export default function OpencodePluginTab() {
  const { t } = useTranslation();
  const [newPlugin, setNewPlugin] = useState("");

  const { data: plugins, isLoading } = useOpencodePlugins();
  const addPlugin = useAddOpencodePlugin();
  const removePlugin = useRemoveOpencodePlugin();
  const reorderPlugins = useReorderOpencodePlugins();
  const importPlugins = useImportOpencodePlugins();

  const handleAdd = async () => {
    const name = newPlugin.trim();
    if (!name) return;
    try {
      await addPlugin.mutateAsync(name);
      setNewPlugin("");
      toast.success(t("common.success"));
    } catch (error) {
      toast.error(t("common.error"), { description: extractErrorMessage(error) });
    }
  };

  const handleRemove = async (normalizedName: string) => {
    try {
      await removePlugin.mutateAsync(normalizedName);
      toast.success(t("common.success"));
    } catch (error) {
      toast.error(t("common.error"), { description: extractErrorMessage(error) });
    }
  };

  const handleMove = async (index: number, direction: -1 | 1) => {
    if (!plugins) return;
    const newIndex = index + direction;
    if (newIndex < 0 || newIndex >= plugins.length) return;

    const newOrder = [...plugins];
    const [item] = newOrder.splice(index, 1);
    newOrder.splice(newIndex, 0, item);

    try {
      await reorderPlugins.mutateAsync(newOrder.map((p) => p.normalized_name));
    } catch (error) {
      toast.error(t("common.error"), { description: extractErrorMessage(error) });
    }
  };

  const handleImport = async () => {
    try {
      const imported = await importPlugins.mutateAsync();
      toast.success(t("plugins.importSuccess", { count: imported.length }));
    } catch (error) {
      toast.error(t("common.error"), { description: extractErrorMessage(error) });
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="text-sm text-muted-foreground">
        {t("plugins.opencode.description")}
      </div>
      <div className="flex items-center gap-2">
        <Input
          placeholder={t("plugins.opencode.addPlaceholder")}
          value={newPlugin}
          onChange={(e) => setNewPlugin(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleAdd()}
          className="flex-1"
        />
        <Button onClick={handleAdd} disabled={!newPlugin.trim()}>
          <Plus className="w-4 h-4 mr-1" />
          {t("common.add")}
        </Button>
        <Button variant="outline" onClick={handleImport}>
          <Download className="w-4 h-4 mr-1" />
          {t("plugins.import")}
        </Button>
      </div>

      {isLoading ? (
        <div className="text-muted-foreground text-sm">{t("common.loading")}</div>
      ) : !plugins || plugins.length === 0 ? (
        <div className="text-center py-12">
          <div className="w-16 h-16 mx-auto mb-4 bg-muted rounded-full flex items-center justify-center">
            <Puzzle className="w-6 h-6 text-muted-foreground" />
          </div>
          <h3 className="text-lg font-medium text-foreground mb-2">
            {t("plugins.opencode.empty")}
          </h3>
          <p className="text-muted-foreground text-sm mb-4">
            {t("plugins.opencode.emptyDescription")}
          </p>
        </div>
      ) : (
        <div className="rounded-xl border border-border-default overflow-hidden">
          {plugins.map((plugin, index) => (
            <div
              key={plugin.normalized_name}
              className={`flex items-center justify-between px-4 py-3 ${
                index !== plugins.length - 1 ? "border-b border-border-default" : ""
              }`}
            >
              <div className="flex flex-col min-w-0 flex-1">
                <span className="text-sm font-medium truncate">
                  {plugin.display_name}
                </span>
                {plugin.display_name !== plugin.normalized_name && (
                  <span className="text-xs text-muted-foreground truncate">
                    {plugin.normalized_name}
                  </span>
                )}
              </div>
              <div className="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8"
                  disabled={index === 0}
                  onClick={() => handleMove(index, -1)}
                >
                  <ArrowUp className="w-4 h-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8"
                  disabled={index === plugins.length - 1}
                  onClick={() => handleMove(index, 1)}
                >
                  <ArrowDown className="w-4 h-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 text-muted-foreground hover:text-red-500"
                  onClick={() => handleRemove(plugin.normalized_name)}
                >
                  <Trash2 className="w-4 h-4" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

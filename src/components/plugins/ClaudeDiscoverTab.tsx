import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Search,
  RefreshCw,
  Download,
  Loader2,
  AlertCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  useClaudeMarketplaceList,
  useInstallClaudePlugin,
  useRefreshClaudeMarketplace,
} from "@/hooks/useClaudeMarketplace";
import { toast } from "sonner";
import { extractErrorMessage } from "@/utils/errorUtils";

function formatInstallCount(count: number): string {
  if (count >= 1000) {
    return `${Math.round(count / 1000)}k`;
  }
  return String(count);
}

export default function ClaudeDiscoverTab() {
  const { t } = useTranslation();
  const { data, isLoading, error } = useClaudeMarketplaceList();
  const installMutation = useInstallClaudePlugin();
  const refreshMutation = useRefreshClaudeMarketplace();
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState<"popular" | "name">("popular");
  const [installingId, setInstallingId] = useState<string | null>(null);

  const installedIds = useMemo(
    () => new Set(data?.installed.map((p) => p.id) ?? []),
    [data?.installed],
  );

  const filteredPlugins = useMemo(() => {
    let list =
      data?.available.filter((p) => !installedIds.has(p.pluginId)) ?? [];
    if (search) {
      const q = search.toLowerCase();
      list = list.filter((p) => p.name.toLowerCase().includes(q));
    }
    if (sort === "popular") {
      list.sort((a, b) => (b.installCount || 0) - (a.installCount || 0));
    } else {
      list.sort((a, b) => a.name.localeCompare(b.name));
    }
    return list;
  }, [data?.available, installedIds, search, sort]);

  const handleInstall = async (pluginId: string, name: string) => {
    setInstallingId(pluginId);
    try {
      await installMutation.mutateAsync(pluginId);
      toast.success(t("plugins.claude.installSuccess", { name }));
    } catch (err) {
      toast.error(t("plugins.claude.installFailed"), {
        description: extractErrorMessage(err),
      });
    } finally {
      setInstallingId(null);
    }
  };

  const handleRefresh = async () => {
    try {
      await refreshMutation.mutateAsync();
      toast.success(t("plugins.claude.refreshSuccess"));
    } catch (err) {
      toast.error(t("plugins.claude.refreshFailed"), {
        description: extractErrorMessage(err),
      });
    }
  };

  if (isLoading) {
    return (
      <div className="text-muted-foreground text-sm">{t("common.loading")}</div>
    );
  }

  if (error) {
    const msg = extractErrorMessage(error);
    const isCliNotFound = msg.includes("not found") || msg.includes("PATH");
    return (
      <div className="text-center py-12">
        <div className="w-16 h-16 mx-auto mb-4 bg-muted rounded-full flex items-center justify-center">
          <AlertCircle className="w-6 h-6 text-muted-foreground" />
        </div>
        <h3 className="text-lg font-medium text-foreground mb-2">
          {isCliNotFound
            ? t("plugins.claude.cliNotFound")
            : t("plugins.claude.discoverError")}
        </h3>
        <p className="text-muted-foreground text-sm">
          {isCliNotFound ? t("plugins.claude.cliNotFoundDescription") : msg}
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder={t("plugins.claude.searchPlaceholder")}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <Select
          value={sort}
          onValueChange={(v) => setSort(v as "popular" | "name")}
        >
          <SelectTrigger className="w-[120px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="popular">
              {t("plugins.claude.sortPopular")}
            </SelectItem>
            <SelectItem value="name">{t("plugins.claude.sortName")}</SelectItem>
          </SelectContent>
        </Select>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleRefresh}
          disabled={refreshMutation.isPending}
        >
          <RefreshCw
            className={`w-4 h-4 ${refreshMutation.isPending ? "animate-spin" : ""}`}
          />
        </Button>
      </div>

      {filteredPlugins.length === 0 ? (
        <div className="text-center py-12">
          <div className="w-16 h-16 mx-auto mb-4 bg-muted rounded-full flex items-center justify-center">
            <Search className="w-6 h-6 text-muted-foreground" />
          </div>
          <h3 className="text-lg font-medium text-foreground mb-2">
            {t("plugins.claude.noResults")}
          </h3>
          <p className="text-muted-foreground text-sm">
            {t("plugins.claude.noResultsDescription")}
          </p>
        </div>
      ) : (
        <div className="rounded-xl border border-border-default overflow-hidden">
          {filteredPlugins.map((plugin, index) => (
            <div
              key={plugin.pluginId}
              className={`flex items-center justify-between px-4 py-3 ${
                index !== filteredPlugins.length - 1
                  ? "border-b border-border-default"
                  : ""
              }`}
            >
              <div className="flex flex-col min-w-0 flex-1 mr-3">
                <span className="text-sm font-medium">{plugin.name}</span>
                <p
                  className="text-xs text-muted-foreground line-clamp-2"
                  title={plugin.description}
                >
                  {plugin.description}
                </p>
                <span className="text-xs text-muted-foreground mt-0.5">
                  <Download className="w-3 h-3 inline mr-1" />
                  {formatInstallCount(plugin.installCount || 0)} installs
                </span>
              </div>
              <Button
                variant="outline"
                size="sm"
                disabled={installingId === plugin.pluginId}
                onClick={() => handleInstall(plugin.pluginId, plugin.name)}
              >
                {installingId === plugin.pluginId ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-1 animate-spin" />
                    {t("plugins.claude.installing")}
                  </>
                ) : (
                  t("plugins.claude.install")
                )}
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

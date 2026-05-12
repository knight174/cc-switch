import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Search,
  RefreshCw,
  Download,
  Loader2,
  AlertCircle,
  Trash2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
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
  useUninstallClaudePlugin,
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
  const uninstallMutation = useUninstallClaudePlugin();
  const refreshMutation = useRefreshClaudeMarketplace();
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState<"popular" | "name">("popular");
  const [filterStatus, setFilterStatus] = useState<
    "all" | "installed" | "uninstalled"
  >("all");
  const [loadingId, setLoadingId] = useState<string | null>(null);

  const installedIds = useMemo(
    () => new Set(data?.installed.map((p) => p.id) ?? []),
    [data?.installed],
  );

  const filteredPlugins = useMemo(() => {
    let list = data?.available ?? [];
    if (search) {
      const q = search.toLowerCase();
      list = list.filter((p) => p.name.toLowerCase().includes(q));
    }
    if (filterStatus === "installed") {
      list = list.filter((p) => installedIds.has(p.pluginId));
    } else if (filterStatus === "uninstalled") {
      list = list.filter((p) => !installedIds.has(p.pluginId));
    }
    if (sort === "popular") {
      list = [...list].sort(
        (a, b) => (b.installCount || 0) - (a.installCount || 0),
      );
    } else {
      list = [...list].sort((a, b) => a.name.localeCompare(b.name));
    }
    return list;
  }, [data?.available, installedIds, search, sort, filterStatus]);

  const handleInstall = async (pluginId: string, name: string) => {
    setLoadingId(pluginId);
    try {
      await installMutation.mutateAsync(pluginId);
      toast.success(t("plugins.claude.installSuccess", { name }));
    } catch (err) {
      toast.error(t("plugins.claude.installFailed"), {
        description: extractErrorMessage(err),
      });
    } finally {
      setLoadingId(null);
    }
  };

  const handleUninstall = async (pluginId: string, name: string) => {
    setLoadingId(pluginId);
    try {
      await uninstallMutation.mutateAsync(pluginId);
      toast.success(t("plugins.claude.uninstallSuccess", { name }));
    } catch (err) {
      toast.error(t("plugins.claude.uninstallFailed"), {
        description: extractErrorMessage(err),
      });
    } finally {
      setLoadingId(null);
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
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error) {
    const msg = extractErrorMessage(error);
    const isCliNotFound = msg.includes("not found") || msg.includes("PATH");
    return (
      <div className="flex flex-col items-center justify-center h-64 text-center">
        <AlertCircle className="h-12 w-12 text-muted-foreground/30 mb-4" />
        <p className="text-lg font-medium text-foreground">
          {isCliNotFound
            ? t("plugins.claude.cliNotFound")
            : t("plugins.claude.discoverError")}
        </p>
        <p className="mt-2 text-sm text-muted-foreground">
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
        <Select
          value={filterStatus}
          onValueChange={(v) =>
            setFilterStatus(v as "all" | "installed" | "uninstalled")
          }
        >
          <SelectTrigger className="w-[140px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("plugins.claude.filterAll")}</SelectItem>
            <SelectItem value="installed">
              {t("plugins.claude.filterInstalled")}
            </SelectItem>
            <SelectItem value="uninstalled">
              {t("plugins.claude.filterNotInstalled")}
            </SelectItem>
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
        <div className="flex flex-col items-center justify-center h-48 text-center">
          <Search className="h-12 w-12 text-muted-foreground/30 mb-4" />
          <p className="text-lg font-medium text-foreground">
            {t("plugins.claude.noResults")}
          </p>
          <p className="mt-2 text-sm text-muted-foreground">
            {t("plugins.claude.noResultsDescription")}
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredPlugins.map((plugin) => {
            const installed = installedIds.has(plugin.pluginId);
            const busy = loadingId === plugin.pluginId;
            return (
              <Card
                key={plugin.pluginId}
                className="glass-card flex flex-col h-full transition-all duration-300 hover:shadow-lg group relative overflow-hidden"
              >
                <div className="absolute inset-0 bg-gradient-to-br from-primary/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none" />
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between gap-2">
                    <CardTitle className="text-base font-semibold truncate flex-1">
                      {plugin.name}
                    </CardTitle>
                    <div className="flex items-center gap-1.5 shrink-0">
                      {(plugin.installCount || 0) > 0 && (
                        <Badge
                          variant="secondary"
                          className="text-[10px] px-1.5 py-0 h-4"
                        >
                          <Download className="h-2.5 w-2.5 mr-0.5" />
                          {formatInstallCount(plugin.installCount || 0)}
                        </Badge>
                      )}
                      {installed && (
                        <Badge
                          variant="default"
                          className="bg-green-600/90 hover:bg-green-600 dark:bg-green-700/90 dark:hover:bg-green-700 text-white border-0 text-[10px] px-1.5 py-0 h-4"
                        >
                          {t("plugins.claude.installedBadge")}
                        </Badge>
                      )}
                    </div>
                  </div>
                </CardHeader>
                <CardContent className="flex-1 pt-0">
                  <p
                    className="text-sm text-muted-foreground/90 line-clamp-3 leading-relaxed"
                    title={plugin.description}
                  >
                    {plugin.description}
                  </p>
                </CardContent>
                <CardFooter className="pt-3 border-t border-border/50 relative z-10">
                  {installed ? (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() =>
                        handleUninstall(plugin.pluginId, plugin.name)
                      }
                      disabled={busy}
                      className="flex-1 border-red-200 text-red-600 hover:bg-red-50 hover:text-red-700 dark:border-red-900/50 dark:text-red-400 dark:hover:bg-red-950/50 dark:hover:text-red-300"
                    >
                      {busy ? (
                        <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
                      ) : (
                        <Trash2 className="h-3.5 w-3.5 mr-1.5" />
                      )}
                      {busy
                        ? t("plugins.claude.uninstalling")
                        : t("plugins.claude.uninstall")}
                    </Button>
                  ) : (
                    <Button
                      variant="mcp"
                      size="sm"
                      onClick={() =>
                        handleInstall(plugin.pluginId, plugin.name)
                      }
                      disabled={busy}
                      className="flex-1"
                    >
                      {busy ? (
                        <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
                      ) : (
                        <Download className="h-3.5 w-3.5 mr-1.5" />
                      )}
                      {busy
                        ? t("plugins.claude.installing")
                        : t("plugins.claude.install")}
                    </Button>
                  )}
                </CardFooter>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}

// Catalog + schema pickers for the create launcher. Shares list hooks with the
// tree (React Query dedupes). Inline "create new" uses mutation hooks directly
// — never dialogs.create(), which would clobber the in-flight launcher request.
import {
  Button,
  Input,
  Label,
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@open-lakehouse/ui-kit";
import {
  parseUcError,
  useCatalogs,
  useCreateCatalog,
  useCreateSchema,
  useSchemas,
} from "@open-lakehouse/unity-catalog-client";
import { Database, FolderTree, Plus } from "lucide-react";
import { useId, useState } from "react";
import { toast } from "sonner";

const CREATE_CATALOG = "__create_catalog__";
const CREATE_SCHEMA = "__create_schema__";

export function CatalogSchemaPicker({
  catalog,
  schema,
  onCatalogChange,
  onSchemaChange,
  requireCatalog = true,
  requireSchema = false,
}: {
  catalog: string;
  schema: string;
  onCatalogChange: (catalog: string) => void;
  onSchemaChange: (schema: string) => void;
  /** When true, catalog must be chosen (schema / leaf creates). */
  requireCatalog?: boolean;
  /** When true, schema must be chosen (volume / model creates). */
  requireSchema?: boolean;
}) {
  const catalogs = useCatalogs();
  const schemas = useSchemas(catalog || undefined);
  const createCatalog = useCreateCatalog();
  const createSchema = useCreateSchema();

  const [inlineCreate, setInlineCreate] = useState<"catalog" | "schema" | null>(
    null,
  );
  const [inlineName, setInlineName] = useState("");

  const catalogId = useId();
  const schemaId = useId();
  const inlineNameId = useId();

  const inlinePending = createCatalog.isPending || createSchema.isPending;

  function submitInlineCreate() {
    const trimmed = inlineName.trim();
    if (!trimmed) return;

    if (inlineCreate === "catalog") {
      createCatalog.mutate(
        { body: { name: trimmed } },
        {
          onSuccess: (data) => {
            const created = data?.name ?? trimmed;
            onCatalogChange(created);
            onSchemaChange("");
            setInlineCreate(null);
            setInlineName("");
            toast.success(`Created catalog "${created}"`);
          },
          onError: (error) => toast.error(parseUcError(error)),
        },
      );
      return;
    }

    if (inlineCreate === "schema" && catalog) {
      createSchema.mutate(
        { body: { name: trimmed, catalog_name: catalog } },
        {
          onSuccess: (data) => {
            const created = data?.name ?? trimmed;
            onSchemaChange(created);
            setInlineCreate(null);
            setInlineName("");
            toast.success(`Created schema "${created}"`);
          },
          onError: (error) => toast.error(parseUcError(error)),
        },
      );
    }
  }

  return (
    <div className="space-y-4">
      {requireCatalog && (
        <div className="space-y-1.5">
          <Label htmlFor={catalogId}>Catalog</Label>
          <Select
            value={catalog || undefined}
            onValueChange={(value) => {
              if (value === CREATE_CATALOG) {
                setInlineCreate("catalog");
                setInlineName("");
                return;
              }
              onCatalogChange(value);
              onSchemaChange("");
            }}
          >
            <SelectTrigger id={catalogId}>
              <SelectValue placeholder="Select a catalog" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {catalogs.data?.map((c) =>
                  c.name ? (
                    <SelectItem key={c.name} value={c.name}>
                      <span className="flex items-center gap-2">
                        <Database className="h-4 w-4 shrink-0 text-muted-foreground" />
                        {c.name}
                      </span>
                    </SelectItem>
                  ) : null,
                )}
              </SelectGroup>
              <SelectGroup>
                <SelectItem value={CREATE_CATALOG} className="text-primary">
                  <span className="flex items-center gap-2">
                    <Plus className="h-4 w-4" />
                    Create a new catalog
                  </span>
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
      )}

      {requireSchema && (
        <div className="space-y-1.5">
          <Label htmlFor={schemaId}>Schema</Label>
          <Select
            value={schema || undefined}
            onValueChange={(value) => {
              if (value === CREATE_SCHEMA) {
                setInlineCreate("schema");
                setInlineName("");
                return;
              }
              onSchemaChange(value);
            }}
            disabled={!catalog}
          >
            <SelectTrigger id={schemaId}>
              <SelectValue placeholder="Select a schema" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {schemas.data?.map((s) =>
                  s.name ? (
                    <SelectItem key={s.name} value={s.name}>
                      <span className="flex items-center gap-2">
                        <FolderTree className="h-4 w-4 shrink-0 text-muted-foreground" />
                        {s.name}
                      </span>
                    </SelectItem>
                  ) : null,
                )}
              </SelectGroup>
              <SelectGroup>
                <SelectItem
                  value={CREATE_SCHEMA}
                  className="text-primary"
                  disabled={!catalog}
                >
                  <span className="flex items-center gap-2">
                    <Plus className="h-4 w-4" />
                    Create a new schema
                  </span>
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
      )}

      {inlineCreate && (
        <div className="space-y-2 rounded-md border bg-muted/30 p-3">
          <Label htmlFor={inlineNameId}>
            {inlineCreate === "catalog"
              ? "New catalog name"
              : "New schema name"}
          </Label>
          <Input
            id={inlineNameId}
            value={inlineName}
            onChange={(e) => setInlineName(e.target.value)}
            placeholder="my_object"
            autoFocus
            disabled={inlinePending}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                submitInlineCreate();
              }
            }}
          />
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={inlinePending}
              onClick={() => {
                setInlineCreate(null);
                setInlineName("");
              }}
            >
              Cancel
            </Button>
            <Button
              type="button"
              size="sm"
              disabled={inlinePending || !inlineName.trim()}
              onClick={submitInlineCreate}
            >
              {inlinePending ? "Creating…" : "Create"}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

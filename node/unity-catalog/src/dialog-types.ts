import type { ObjectKind, StorageKind } from "./types";

/** Catalog-namespace create requests handled by CreateEntityDialog. */
export type CreateRequest =
  | { kind: "catalog" }
  | { kind: "schema"; catalog: string }
  | { kind: "volume"; catalog: string; schema: string }
  | { kind: "model"; catalog: string; schema: string };

/** Metastore-level create requests handled by the storage dialogs. */
export type StorageCreateRequest = { kind: StorageKind };

/** Anything the dialogs provider's `create` accepts. */
export type AnyCreateRequest = CreateRequest | StorageCreateRequest;

/** Catalog-namespace entities that support PATCH (comment / rename). */
export type EditableKind = "catalog" | "schema" | "volume" | "model";

export interface EditRequest {
  kind: EditableKind;
  /** Catalog: name; everything else: fully-qualified name. */
  name: string;
  comment?: string;
}

/** Edit request for a metastore-level storage securable. */
export interface StorageEditRequest {
  kind: StorageKind;
  name: string;
}

/** Anything the dialogs provider's `edit` accepts. */
export type AnyEditRequest = EditRequest | StorageEditRequest;

/** Entities that support DELETE. */
export type DeletableKind = "catalog" | "schema" | ObjectKind | StorageKind;

export interface DeleteRequest {
  kind: DeletableKind;
  name: string;
}

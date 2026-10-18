// Catalog dialog orchestration.
//
// Tree rows and the detail pane trigger create/edit/delete flows through this
// context instead of threading callbacks down the tree. The provider owns the
// transient dialog request state and renders the matching dialog. It routes the
// metastore-level storage securables (credentials, external locations) to their
// dedicated dialogs and everything else to the generic catalog dialogs.
import {
  createContext,
  type ReactNode,
  useContext,
  useMemo,
  useState,
} from "react";

import { CreateEntityDialog } from "./CreateEntityDialog";
import { DeleteEntityDialog } from "./DeleteEntityDialog";
import type {
  AnyCreateRequest,
  AnyEditRequest,
  DeleteRequest,
  EditRequest,
} from "./dialog-types";
import { isCatalogCreateRequest } from "./dialog-types";
import { EditEntityDialog } from "./EditEntityDialog";
import { CredentialDialog } from "./storage/CredentialDialog";
import { ExternalLocationDialog } from "./storage/ExternalLocationDialog";

export type {
  AnyCreateRequest,
  AnyEditRequest,
  CatalogCreateRequest,
  CreateRequest,
  DeletableKind,
  DeleteRequest,
  EditableKind,
  EditRequest,
  StorageCreateRequest,
  StorageEditRequest,
} from "./dialog-types";

interface CatalogDialogsValue {
  create: (req: AnyCreateRequest) => void;
  edit: (req: AnyEditRequest) => void;
  remove: (req: DeleteRequest) => void;
}

const CatalogDialogsContext = createContext<CatalogDialogsValue | undefined>(
  undefined,
);

export function CatalogDialogsProvider({ children }: { children: ReactNode }) {
  const [createReq, setCreateReq] = useState<AnyCreateRequest>();
  const [editReq, setEditReq] = useState<AnyEditRequest>();
  const [deleteReq, setDeleteReq] = useState<DeleteRequest>();

  const value = useMemo<CatalogDialogsValue>(
    () => ({
      create: setCreateReq,
      edit: setEditReq,
      remove: setDeleteReq,
    }),
    [],
  );

  const closeCreate = () => setCreateReq(undefined);
  const closeEdit = () => setEditReq(undefined);

  return (
    <CatalogDialogsContext.Provider value={value}>
      {children}

      {createReq?.kind === "credential" && (
        <CredentialDialog mode="create" onClose={closeCreate} />
      )}
      {createReq?.kind === "external_location" && (
        <ExternalLocationDialog mode="create" onClose={closeCreate} />
      )}
      {createReq && isCatalogCreateRequest(createReq) && (
        <CreateEntityDialog request={createReq} onClose={closeCreate} />
      )}

      {editReq?.kind === "credential" && (
        <CredentialDialog mode="edit" name={editReq.name} onClose={closeEdit} />
      )}
      {editReq?.kind === "external_location" && (
        <ExternalLocationDialog
          mode="edit"
          name={editReq.name}
          onClose={closeEdit}
        />
      )}
      {editReq &&
        editReq.kind !== "credential" &&
        editReq.kind !== "external_location" && (
          <EditEntityDialog
            request={editReq as EditRequest}
            onClose={closeEdit}
          />
        )}

      {deleteReq && (
        <DeleteEntityDialog
          request={deleteReq}
          onClose={() => setDeleteReq(undefined)}
        />
      )}
    </CatalogDialogsContext.Provider>
  );
}

export function useCatalogDialogs(): CatalogDialogsValue {
  const ctx = useContext(CatalogDialogsContext);
  if (!ctx) {
    throw new Error(
      "useCatalogDialogs must be used within a CatalogDialogsProvider",
    );
  }
  return ctx;
}

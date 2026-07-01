import { useCredentialDetail } from "../uc/queries";

import { DetailStates } from "./DetailStates";
import { Meta, MetaGrid } from "./Meta";

export function CredentialDetail({ name }: { name: string }) {
  const { data: credential, isLoading, error } = useCredentialDetail(name);
  if (!credential) return <DetailStates isLoading={isLoading} error={error} />;

  return (
    <MetaGrid>
      <Meta label="Purpose" value={credential.purpose} />
      <Meta label="Owner" value={credential.owner} />
      <Meta label="Created by" value={credential.created_by} />
      <Meta
        label="IAM role ARN"
        value={credential.aws_iam_role?.role_arn}
        wide
        mono
      />
      <Meta
        label="Unity Catalog IAM ARN"
        value={credential.aws_iam_role?.unity_catalog_iam_arn}
        wide
        mono
      />
      <Meta
        label="External ID"
        value={credential.aws_iam_role?.external_id}
        wide
        mono
      />
      <Meta label="Comment" value={credential.comment} wide />
    </MetaGrid>
  );
}

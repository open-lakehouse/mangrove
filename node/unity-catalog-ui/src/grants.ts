// The `@open-lakehouse/unity-catalog-ui/grants` entry point: the grant fields
// without the catalog browser, for hosts that render one grant form and must
// not pull in the rest of the package (the editor, the Delta log viewer). The
// MCP App in @open-lakehouse/uc-a2ui is one: it ships as a single inlined
// HTML file. Needs only @open-lakehouse/ui-kit and the UC client.

export {
  canSubmitGrant,
  GrantForm,
  type GrantFormSubmission,
  type GrantFormValue,
  grantSubmission,
} from "./detail/GrantForm.js";

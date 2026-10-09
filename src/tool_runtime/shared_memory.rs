//! Automatic shared background on successful, exact Workflow Session results.
//! No opt-in context_request, inferred Session, native-memory mutation or Host claim.

use super::{ToolResult, ToolRuntime};
use crate::auth::AuthContext;
use serde_json::{json, Value};

impl ToolRuntime {
    pub(crate) async fn attach_shared_memory_background(
        &self,
        result: &mut ToolResult,
        auth: Option<&AuthContext>,
    ) {
        if !result.success {
            return;
        }
        let Some(session_id) = result
            .output
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            return;
        };
        let Some(project_id) = result
            .output
            .get("resolved_project")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            return;
        };
        let unavailable = |reason: &str| {
            json!({"status": "unavailable", "reason_code": reason,
            "boundary": "server-tool-result", "host_request_verified": false})
        };
        if self
            .authorize_session_target(&session_id, "work_on_project", auth)
            .await
            .is_err()
        {
            result.output["shared_memory"] = unavailable("shared_memory_session_unavailable");
            return;
        }
        let resolved = match self.resolve_project_input_for_auth(&project_id, auth).await {
            Ok(resolved) => resolved,
            Err(_) => {
                result.output["shared_memory"] = unavailable("shared_memory_project_unavailable");
                return;
            }
        };
        let session = self.sessions.summary(&session_id, Some(0));
        if session
            .as_ref()
            .and_then(|session| session.project.as_deref())
            != Some(resolved.resolved_id.as_str())
        {
            result.output["shared_memory"] = unavailable("shared_memory_session_scope_mismatch");
            return;
        }
        result.output["shared_memory"] = crate::mcp_gateway::shared_memory_background(
            self,
            &resolved.config.client_id,
            &resolved.config.path,
            &session_id,
            auth,
        )
        .await;
    }
}

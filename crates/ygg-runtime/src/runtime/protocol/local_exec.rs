use super::*;

const DEPLOYMENT_HUB_SESSION_ID: &str = "kernel_deployment_hub";

impl<S> Runtime<S>
where
    S: EventStore,
{
    // --- Target ---

    pub(crate) async fn dispatch_target_list(&self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(
            self.config.target_registry.list().await,
        )?)
    }

    pub(crate) async fn dispatch_target_status(&self, params: &Value) -> anyhow::Result<Value> {
        let target_id = required_str(params, "target_id", "kernel.v1.target.status")?;
        Ok(serde_json::to_value(
            self.config
                .target_registry
                .status(&target_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("execution target '{target_id}' not found"))?,
        )?)
    }

    pub(crate) async fn dispatch_target_register(&self, params: Value) -> anyhow::Result<Value> {
        let target: crate::runtime::ExecutionTarget = serde_json::from_value(params)?;
        self.config.target_registry.register(target.clone()).await;
        Ok(serde_json::to_value(target)?)
    }

    pub(crate) async fn dispatch_target_unregister(&self, params: &Value) -> anyhow::Result<Value> {
        let target_id = required_str(params, "target_id", "kernel.v1.target.unregister")?;
        Ok(serde_json::to_value(
            self.config
                .target_registry
                .unregister(&target_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("execution target '{target_id}' not found"))?,
        )?)
    }

    // --- Exec ---

    pub(crate) async fn dispatch_exec_start(
        &self,
        context: &ProtocolContext,
        params: Value,
    ) -> anyhow::Result<Value> {
        let request: crate::runtime::LocalExecStartRequest = serde_json::from_value(params)?;
        self.append_deployment_hub_event(
            context,
            ygg_core::EVENT_EXEC_REQUEST,
            json!({
                "target_id": request.target_id,
                "program": request.command.program,
                "arg_count": request.command.args.len(),
                "port_names": request.port_names,
            }),
        )
        .await?;

        let response = self
            .config
            .local_exec_executor
            .executor()
            .start(request)
            .await?;
        self.config
            .exec_registry
            .upsert_from_status(&response.status)
            .await;

        let event_kind = if response.status.kind == crate::runtime::ExecStatusKind::Denied {
            ygg_core::EVENT_EXEC_DENIED
        } else {
            ygg_core::EVENT_EXEC_STARTED
        };
        self.append_deployment_hub_event(
            context,
            event_kind,
            json!({
                "exec_id": response.exec_id,
                "target_id": response.status.target_id,
                "status": response.status.kind,
                "ready": response.status.ready,
                "error": response.error,
            }),
        )
        .await?;

        Ok(serde_json::to_value(response)?)
    }

    pub(crate) async fn dispatch_exec_stop(
        &self,
        context: &ProtocolContext,
        params: Value,
    ) -> anyhow::Result<Value> {
        let request: crate::runtime::LocalExecStopRequest = serde_json::from_value(params)?;
        if request.exec_id.trim().is_empty() {
            anyhow::bail!("kernel.v1.exec.stop requires exec_id");
        }
        let response = self
            .config
            .local_exec_executor
            .executor()
            .stop(request)
            .await?;
        self.config
            .exec_registry
            .upsert_from_status(&response.status)
            .await;

        self.append_deployment_hub_event(
            context,
            ygg_core::EVENT_EXEC_STOPPED,
            json!({
                "exec_id": response.exec_id,
                "status": response.status.kind,
                "error": response.error,
            }),
        )
        .await?;

        Ok(serde_json::to_value(response)?)
    }

    pub(crate) async fn dispatch_exec_status(&self, params: Value) -> anyhow::Result<Value> {
        let request: crate::runtime::LocalExecStatusRequest = serde_json::from_value(params)?;
        if request.exec_id.trim().is_empty() {
            anyhow::bail!("kernel.v1.exec.status requires exec_id");
        }
        let response = self
            .config
            .local_exec_executor
            .executor()
            .status(request)
            .await?;
        self.config
            .exec_registry
            .upsert_from_status(&response.status)
            .await;
        Ok(serde_json::to_value(response)?)
    }

    pub(crate) async fn dispatch_exec_logs(&self, params: Value) -> anyhow::Result<Value> {
        let request: crate::runtime::LocalExecLogsRequest = serde_json::from_value(params)?;
        if request.exec_id.trim().is_empty() {
            anyhow::bail!("kernel.v1.exec.logs requires exec_id");
        }
        Ok(serde_json::to_value(
            self.config
                .local_exec_executor
                .executor()
                .logs(request)
                .await?,
        )?)
    }

    pub(crate) async fn dispatch_exec_list(&self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(
            crate::runtime::LocalExecListResponse {
                executions: self.config.exec_registry.list().await,
            },
        )?)
    }

    // --- Port ---

    pub(crate) async fn dispatch_port_lease(
        &self,
        context: &ProtocolContext,
        params: Value,
    ) -> anyhow::Result<Value> {
        let request: crate::runtime::PortLeaseRequest = serde_json::from_value(params)?;
        let response = self.config.port_lease_registry.lease(request).await;
        self.append_deployment_hub_event(
            context,
            ygg_core::EVENT_PORT_LEASED,
            json!({
                "lease_id": response.lease.id,
                "target_id": response.lease.target_id,
                "port_name": response.lease.port_name,
                "host": response.lease.host,
                "port": response.lease.port,
                "protocol": response.lease.protocol,
                "bind": response.lease.bind,
                "status": response.lease.status,
            }),
        )
        .await?;
        Ok(serde_json::to_value(response)?)
    }

    pub(crate) async fn dispatch_port_release(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let lease_id = required_str(params, "lease_id", "kernel.v1.port.release")?;
        let lease = self
            .config
            .port_lease_registry
            .release(&lease_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("port lease '{lease_id}' not found"))?;
        self.append_deployment_hub_event(
            context,
            ygg_core::EVENT_PORT_RELEASED,
            json!({
                "lease_id": lease.id,
                "target_id": lease.target_id,
                "port_name": lease.port_name,
                "host": lease.host,
                "port": lease.port,
                "status": lease.status,
            }),
        )
        .await?;
        Ok(serde_json::to_value(lease)?)
    }

    pub(crate) async fn dispatch_port_status(&self, params: &Value) -> anyhow::Result<Value> {
        let lease_id = required_str(params, "lease_id", "kernel.v1.port.status")?;
        Ok(serde_json::to_value(
            self.config
                .port_lease_registry
                .status(&lease_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("port lease '{lease_id}' not found"))?,
        )?)
    }

    pub(crate) async fn dispatch_port_list(&self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(
            self.config.port_lease_registry.list().await,
        )?)
    }

    // --- Proxy ---

    pub(crate) async fn dispatch_proxy_register(
        &self,
        context: &ProtocolContext,
        params: Value,
    ) -> anyhow::Result<Value> {
        let request: crate::runtime::ProxyRouteRegisterRequest = serde_json::from_value(params)?;
        let lease_id = request.upstream.port_lease_id.clone();
        let lease = self.config.port_lease_registry.status(&lease_id).await;
        if !matches!(
            lease.as_ref().map(|lease| lease.status),
            Some(crate::runtime::PortLeaseStatusKind::Active)
        ) {
            self.append_deployment_hub_event(
                context,
                ygg_core::EVENT_PROXY_DENIED,
                json!({
                    "port_lease_id": lease_id,
                    "reason": "missing_or_inactive_port_lease",
                }),
            )
            .await?;
            anyhow::bail!(
                "kernel.v1.proxy.register requires an existing active port lease upstream"
            );
        }
        let lease = lease.expect("active lease checked above");
        if request.upstream.port_name != lease.port_name {
            self.append_deployment_hub_event(
                context,
                ygg_core::EVENT_PROXY_DENIED,
                json!({
                    "port_lease_id": lease_id,
                    "requested_port_name": request.upstream.port_name,
                    "leased_port_name": lease.port_name,
                    "reason": "port_name_mismatch",
                }),
            )
            .await?;
            anyhow::bail!(
                "kernel.v1.proxy.register upstream port_name must match the referenced port lease"
            );
        }

        let response = self.config.proxy_route_registry.register(request).await;
        self.append_deployment_hub_event(
            context,
            ygg_core::EVENT_PROXY_REGISTERED,
            json!({
                "route_id": response.route.id,
                "port_lease_id": response.route.upstream.port_lease_id,
                "port_name": response.route.upstream.port_name,
                "protocol": response.route.protocol,
                "public_url": response.route.public_url,
                "iframe_url": response.route.iframe_url,
                "status": response.route.status,
            }),
        )
        .await?;
        Ok(serde_json::to_value(response)?)
    }

    pub(crate) async fn dispatch_proxy_unregister(
        &self,
        context: &ProtocolContext,
        params: &Value,
    ) -> anyhow::Result<Value> {
        let route_id = required_str(params, "route_id", "kernel.v1.proxy.unregister")?;
        let route = self
            .config
            .proxy_route_registry
            .unregister(&route_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("proxy route '{route_id}' not found"))?;
        self.append_deployment_hub_event(
            context,
            ygg_core::EVENT_PROXY_UNREGISTERED,
            json!({
                "route_id": route.id,
                "port_lease_id": route.upstream.port_lease_id,
                "status": route.status,
            }),
        )
        .await?;
        Ok(serde_json::to_value(route)?)
    }

    pub(crate) async fn dispatch_proxy_status(&self, params: &Value) -> anyhow::Result<Value> {
        let route_id = required_str(params, "route_id", "kernel.v1.proxy.status")?;
        Ok(serde_json::to_value(
            self.config
                .proxy_route_registry
                .status(&route_id)
                .await
                .ok_or_else(|| anyhow::anyhow!("proxy route '{route_id}' not found"))?,
        )?)
    }

    pub(crate) async fn dispatch_proxy_list(&self) -> anyhow::Result<Value> {
        Ok(serde_json::to_value(
            self.config.proxy_route_registry.list().await,
        )?)
    }

    async fn append_deployment_hub_event(
        &self,
        context: &ProtocolContext,
        kind: &'static str,
        payload: Value,
    ) -> anyhow::Result<()> {
        let session_id = if let Some(session_id) = context.session_id.as_deref() {
            session_id.to_string()
        } else {
            self.ensure_deployment_hub_session().await?
        };
        self.append_kernel_event(&session_id, kind, payload).await?;
        Ok(())
    }

    async fn ensure_deployment_hub_session(&self) -> anyhow::Result<String> {
        {
            let sessions = self.sessions.read().await;
            if matches!(
                sessions
                    .get(DEPLOYMENT_HUB_SESSION_ID)
                    .map(|session| &session.status),
                Some(ygg_core::SessionStatus::Open)
            ) {
                return Ok(DEPLOYMENT_HUB_SESSION_ID.to_string());
            }
        }

        let now = chrono::Utc::now();
        let mut sessions = self.sessions.write().await;
        sessions.insert(
            DEPLOYMENT_HUB_SESSION_ID.to_string(),
            ygg_core::KernelSession {
                id: DEPLOYMENT_HUB_SESSION_ID.to_string(),
                labels: vec!["kernel".to_string(), "deployment_hub".to_string()],
                active_package_set: Vec::new(),
                principal_scope: None,
                status: ygg_core::SessionStatus::Open,
                created_at: now,
                updated_at: now,
                metadata: json!({"synthetic": true}),
            },
        );
        Ok(DEPLOYMENT_HUB_SESSION_ID.to_string())
    }
}

fn required_str(params: &Value, field: &str, method: &str) -> anyhow::Result<String> {
    let value = params
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{method} requires {field}"))?;
    if value.trim().is_empty() {
        anyhow::bail!("{method} requires {field}");
    }
    Ok(value.to_string())
}

use std::collections::HashSet;
use std::future::Future;
use std::time::Duration;

use futures::channel::oneshot;
use futures::future::{self, Either};
use instant::Instant;
use uuid::Uuid;
use warp_core::features::FeatureFlag;
use warpui::r#async::{FutureExt, TimeoutError};
use warpui::{Entity, ModelContext, ModelDropped, ModelHandle, ModelSpawner, SingletonEntity};

use crate::ai::mcp::file_based_manager::FileBasedMCPManagerEvent;
use crate::ai::mcp::templatable_manager::TemplatableMCPServerManagerEvent;
use crate::ai::mcp::{FileBasedMCPManager, MCPServerState, TemplatableMCPServerManager};

#[derive(Debug, Eq, PartialEq)]
enum ModelEventWaitError {
    SubscriptionDropped,
    TimedOut,
}

/// 同一调用方对同一模型的等待必须串行；取消订阅会移除该调用方的全部订阅。
/// 超时清理完成后才返回，避免旧回调移除下一次等待的订阅。
fn await_model_event<M, S, T, F>(
    handle: &ModelHandle<S>,
    timeout: Duration,
    ctx: &mut ModelContext<M>,
    mut on_event: F,
) -> impl Future<Output = Result<T, ModelEventWaitError>> + use<M, S, T, F>
where
    M: Entity,
    S: Entity,
    S::Event: 'static,
    T: Send + 'static,
    F: FnMut(&S::Event, &mut ModelContext<M>) -> Option<T> + 'static,
{
    let (tx, rx) = oneshot::channel();
    let mut tx = Some(tx);
    ctx.unsubscribe_from_model(handle);
    ctx.subscribe_to_model(handle, move |_, handle, event, ctx| {
        let Some(value) = on_event(event, ctx) else {
            return;
        };
        if let Some(sender) = tx.take() {
            let _ = sender.send(value);
        }
        ctx.unsubscribe_from_model(&handle);
    });

    let foreground = ctx.spawner();
    let handle = handle.clone();
    async move {
        match rx.with_timeout(timeout).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(oneshot::Canceled)) => Err(ModelEventWaitError::SubscriptionDropped),
            Err(TimeoutError) => {
                let _ = foreground
                    .spawn(move |_, ctx| ctx.unsubscribe_from_model(&handle))
                    .await;
                Err(ModelEventWaitError::TimedOut)
            }
        }
    }
}

/// 初次全局扫描和服务就绪共用一个有界预算。仅等待实际自动启动的服务，
/// 不代替用户授权，也不把可恢复的 MCP 启动失败提升为 Agent 运行失败。
pub(super) async fn await_initial_global_startup<M: Entity>(
    foreground: &ModelSpawner<M>,
    timeout: Duration,
) -> Result<(), ModelDropped> {
    if !FeatureFlag::FileBasedMcp.is_enabled() {
        return Ok(());
    }
    let deadline = Instant::now() + timeout;
    let uuids = foreground
        .spawn(move |_, ctx| {
            wait_for_initial_global_scan(deadline.saturating_duration_since(Instant::now()), ctx)
        })
        .await?
        .await;
    let remaining = deadline.saturating_duration_since(Instant::now());
    if !uuids.is_empty() && !remaining.is_zero() {
        foreground
            .spawn(move |_, ctx| {
                wait_for_file_based_mcps_running(
                    uuids,
                    deadline.saturating_duration_since(Instant::now()),
                    ctx,
                )
            })
            .await?
            .await;
    }
    Ok(())
}

fn wait_for_initial_global_scan<M: Entity>(
    timeout: Duration,
    ctx: &mut ModelContext<M>,
) -> impl Future<Output = Vec<Uuid>> + use<M> {
    let manager = FileBasedMCPManager::handle(ctx);
    if let Some(uuids) = manager.as_ref(ctx).initial_global_scan_result() {
        return Either::Right(future::ready(uuids));
    }
    let wait = await_model_event(&manager, timeout, ctx, |event, _| {
        if let FileBasedMCPManagerEvent::InitialGlobalMcpScanComplete { wait_server_uuids } = event
        {
            Some(wait_server_uuids.clone())
        } else {
            None
        }
    });
    Either::Left(async move {
        match wait.await {
            Ok(uuids) => uuids,
            Err(ModelEventWaitError::SubscriptionDropped | ModelEventWaitError::TimedOut) => {
                log::warn!("Initial global MCP scan did not settle in time; proceeding");
                Vec::new()
            }
        }
    })
}

fn wait_for_file_based_mcps_running<M: Entity>(
    uuids: Vec<Uuid>,
    timeout: Duration,
    ctx: &mut ModelContext<M>,
) -> impl Future<Output = ()> + use<M> {
    let file_manager = FileBasedMCPManager::as_ref(ctx);
    let manager = TemplatableMCPServerManager::as_ref(ctx);
    let mut pending: HashSet<_> = uuids
        .into_iter()
        .filter(|uuid| {
            file_manager.get_hash_by_uuid(*uuid).is_some()
                && !matches!(
                    manager.get_server_state(*uuid),
                    Some(
                        MCPServerState::Running
                            | MCPServerState::FailedToStart
                            | MCPServerState::NotRunning
                    )
                )
        })
        .collect();
    if pending.is_empty() {
        return Either::Right(future::ready(()));
    }

    let manager = TemplatableMCPServerManager::handle(ctx);
    let wait = await_model_event(&manager, timeout, ctx, move |event, _| {
        let TemplatableMCPServerManagerEvent::StateChanged { uuid, state } = event else {
            return None;
        };
        match state {
            MCPServerState::Running
            | MCPServerState::FailedToStart
            | MCPServerState::NotRunning => {
                pending.remove(uuid);
            }
            MCPServerState::Starting
            | MCPServerState::Authenticating
            | MCPServerState::ShuttingDown => {}
        }
        pending.is_empty().then_some(())
    });
    Either::Left(async move {
        if let Err(error) = wait.await {
            log::warn!(
                "Global MCP readiness wait ended before all servers settled: {error:?}; proceeding"
            );
        }
    })
}

#[cfg(test)]
#[path = "mcp_startup_tests.rs"]
mod tests;

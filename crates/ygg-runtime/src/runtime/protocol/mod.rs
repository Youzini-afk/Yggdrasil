use bytes::Bytes;
use reqwest::header::{HeaderName, HeaderValue};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use ygg_core::{
    project::{ProjectId, ProjectState},
    CapHandleId, PackageId, RedactionState,
};

use super::{OpenSessionRequest, Runtime};
use crate::{
    EventListRequest, EventStore, OutboundFrameKind, OutboundStreamFrame, OutboundWebSocketFrame,
    ProtocolContext, ProtocolPrincipal, StreamEmitter, StreamRegistry, WebSocketEvent,
};

mod surface;
mod projects;
mod outbound_dispatch;
mod audit;
mod permissions;
mod proposals;
mod sessions_events;
mod packages;
mod capabilities;
mod assets_projections;

#[cfg(test)]
mod tests;

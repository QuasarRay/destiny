//! Authenticated Carbon/Lightyear interoperability for the Destiny facade.
//!
//! A product host authenticates a Lightyear link and inserts
//! [`CarbonClientIdentity`] on that link. Carbon recipient identifiers are
//! resolved only through those authenticated components. Unknown or duplicate
//! identifiers fail closed: the complete source envelope is rejected and no
//! recipient is sent a frame.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{self, Write},
    panic::{AssertUnwindSafe, catch_unwind},
};

use avian3d::prelude::{
    AngularVelocity, Collider, ColliderDisabled, GravityScale, LinearVelocity, Mass,
    MaxAngularSpeed, MaxLinearSpeed, Position, RigidBody, Rotation,
};
use bevy::prelude::{
    App, Bundle, Commands, Component, Entity, IntoScheduleConfigs, Plugin, PostUpdate, PreUpdate,
    Query, Res, ResMut, Resource,
};
use bevy_replicon::prelude::AppRuleExt;
use lightyear_connection::prelude::NetworkDirection;
use lightyear_messages::prelude::{
    AppMessageExt, MessageReceiver, MessageSender, MessageSystems,
};
use lightyear_transport::prelude::{AppChannelExt, ChannelMode, ChannelSettings, ReliableSettings};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    carbon_codec::decode_canonical,
    CarbonNetworkOutbox, DestinyBallId, DestinyBallMetadata, DestinyMass,
    DestinyPendingRemoval,
};

pub use bevy_replicon::prelude::Replicated;
pub use lightyear_replication::{
    LightyearRepliconBackend, LightyearRepliconServerBackend,
    prelude::Replicate,
    visibility::room::{RoomAllocator, RoomId, RoomPlugin, Rooms},
};

pub const CARBON_PROTOCOL_NAME: &str = "destiny-carbon-update";
pub const CARBON_SCHEMA_VERSION: u16 = 2;
const DEFAULT_MAX_WIRE_MESSAGES_PER_FLUSH: usize = 100_000;
const DEFAULT_MAX_WIRE_BYTES_PER_FLUSH: usize = 48 * 1024 * 1024;
const DEFAULT_MAX_CLIENT_INBOX_MESSAGES: usize = 10_000;
const DEFAULT_MAX_CLIENT_INBOX_BYTES: usize = 48 * 1024 * 1024;
const DEDUP_WINDOW: usize = 4_096;
const MAX_EXPANDED_UPDATE_ROWS: usize = 200_000;
const MAX_VISIBILITY_ROOMS: usize = 60_000;
const MAX_BUBBLES_PER_CLIENT: usize = 4_096;

/// Ordered-reliable Lightyear channel for one recipient's Carbon tick frame.
pub struct CarbonUpdateChannel;

/// An authenticated product identity attached to exactly one Lightyear link.
///
/// The compatibility plugin does not authenticate accounts. A host must add
/// this component only after its own authentication and park-authorization
/// checks have succeeded.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarbonClientIdentity(pub i64);

/// Bubble subscriptions attached to an authenticated client link.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct CarbonClientBubbles(pub Vec<i64>);

/// Optional owner of a cloaked ball. Cloaked replication is owner-only.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarbonBallOwner(pub i64);

/// One idempotent, recipient-specific wire frame.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CarbonUpdateMessage {
    pub protocol: String,
    pub schema_version: u16,
    pub batch_id: i64,
    pub recipient_id: i64,
    pub updates: Vec<Value>,
}

impl CarbonUpdateMessage {
    fn validate(&self) -> Result<(), String> {
        if self.protocol != CARBON_PROTOCOL_NAME {
            return Err(format!("unsupported Carbon protocol {:?}", self.protocol));
        }
        if self.schema_version != CARBON_SCHEMA_VERSION {
            return Err(format!(
                "unsupported Carbon schema version {}",
                self.schema_version
            ));
        }
        if self.batch_id <= 0 {
            return Err("Carbon batch identifier must be positive".into());
        }
        if self.updates.len() > MAX_EXPANDED_UPDATE_ROWS {
            return Err("Carbon delivery exceeds the update-row limit".into());
        }
        // Validate the complete canonical tree before it can enter the client
        // inbox. In particular, a correct outer recipient must not be allowed
        // to conceal update rows addressed to another Carbon identity.
        let decoded = decode_canonical(Value::Array(self.updates.clone()))?;
        let rows = decoded
            .as_array()
            .ok_or_else(|| "Carbon delivery updates must be an array".to_owned())?;
        for row in rows {
            let row = row
                .as_array()
                .ok_or_else(|| "Carbon delivery row must be an array".to_owned())?;
            if row.len() < 3
                || row.first().and_then(Value::as_i64) != Some(self.recipient_id)
                || !row
                    .get(1)
                    .and_then(Value::as_str)
                    .is_some_and(|action| !action.is_empty())
                || !row.get(2).is_some_and(Value::is_array)
            {
                return Err("Carbon delivery row has an invalid recipient or action".into());
            }
        }
        Ok(())
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct CarbonTransportLimits {
    pub max_wire_messages_per_flush: usize,
    pub max_wire_bytes_per_flush: usize,
    pub max_client_inbox_messages: usize,
    pub max_client_inbox_bytes: usize,
}

impl Default for CarbonTransportLimits {
    fn default() -> Self {
        Self {
            max_wire_messages_per_flush: DEFAULT_MAX_WIRE_MESSAGES_PER_FLUSH,
            max_wire_bytes_per_flush: DEFAULT_MAX_WIRE_BYTES_PER_FLUSH,
            max_client_inbox_messages: DEFAULT_MAX_CLIENT_INBOX_MESSAGES,
            max_client_inbox_bytes: DEFAULT_MAX_CLIENT_INBOX_BYTES,
        }
    }
}

/// Observable transport/backpressure counters. Counters saturate rather than
/// wrapping during a long-running server process.
#[derive(Resource, Clone, Debug, Default)]
pub struct CarbonTransportMetrics {
    pub delivered_frames: u64,
    pub delivered_bytes: u64,
    pub duplicate_bindings: u64,
    pub unknown_recipients: u64,
    pub rejected_envelopes: u64,
    pub backpressure_events: u64,
    pub duplicate_frames: u64,
    pub unauthorized_frames: u64,
    pub room_allocation_failures: u64,
}

#[derive(Resource, Clone, Debug, Default)]
struct CarbonClientBindings {
    by_id: HashMap<i64, Entity>,
    invalid_ids: HashSet<i64>,
}

#[derive(Resource, Debug, Default)]
struct CarbonVisibilityRooms {
    global: Option<RoomId>,
    bubbles: HashMap<i64, RoomId>,
    personal: HashMap<i64, RoomId>,
}

/// Bounded, deduplicated receive queue for a native Carbon client bridge.
/// Product code drains this queue and passes each frame's `updates` to the
/// Carbon `destiny.net.client.Ticker`. A reconnect must request and apply a
/// fresh `SetState` before normal updates are accepted again.
#[derive(Resource, Debug, Default)]
pub struct CarbonClientInbox {
    frames: VecDeque<CarbonUpdateMessage>,
    queued_bytes: usize,
    seen_order: VecDeque<(i64, i64)>,
    seen: HashSet<(i64, i64)>,
    last_batch_by_recipient: HashMap<i64, i64>,
    requires_rebase: bool,
}

impl CarbonClientInbox {
    pub fn pop_front(&mut self) -> Option<CarbonUpdateMessage> {
        let frame = self.frames.pop_front()?;
        self.queued_bytes = self.queued_bytes.saturating_sub(encoded_len(&frame));
        Some(frame)
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn requires_rebase(&self) -> bool {
        self.requires_rebase
    }

    pub fn request_rebase(&mut self) {
        self.frames.clear();
        self.queued_bytes = 0;
        self.seen.clear();
        self.seen_order.clear();
        // Keep the monotonic watermark across a rebase. Clearing it would let
        // a delayed pre-reconnect frame become valid again immediately after
        // the host acknowledges its fresh full-state replacement.
        self.requires_rebase = true;
    }

    /// Called only after the product ticker has successfully applied a fresh,
    /// authenticated full-state replacement through `through_batch_id`.
    ///
    /// Recording that watermark is mandatory. Merely clearing the rebase flag
    /// would allow a delayed frame from before the replacement to be accepted
    /// immediately afterwards.
    pub fn acknowledge_rebase(
        &mut self,
        recipient_id: i64,
        through_batch_id: i64,
    ) -> Result<(), String> {
        if !self.requires_rebase {
            return Err("the Carbon inbox is not awaiting a rebase".into());
        }
        if through_batch_id <= 0
            || self
                .last_batch_by_recipient
                .get(&recipient_id)
                .is_some_and(|last| through_batch_id <= *last)
        {
            return Err("the rebase watermark must advance the recipient batch sequence".into());
        }
        self.last_batch_by_recipient
            .insert(recipient_id, through_batch_id);
        let watermark = (recipient_id, through_batch_id);
        self.seen.insert(watermark);
        self.seen_order.push_back(watermark);
        while self.seen_order.len() > DEDUP_WINDOW {
            if let Some(expired) = self.seen_order.pop_front() {
                self.seen.remove(&expired);
            }
        }
        self.requires_rebase = false;
        Ok(())
    }
}

/// Registers authoritative state, typed Carbon frames, relevance rooms, a
/// recipient-specific send path, and a bounded client receive path. Add the
/// appropriate Lightyear client/server plugins and [`RoomPlugin`] first.
#[derive(Default)]
pub struct DestinyCarbonInteropPlugin;

impl Plugin for DestinyCarbonInteropPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CarbonNetworkOutbox>();
        app.init_resource::<CarbonClientBindings>();
        app.init_resource::<CarbonVisibilityRooms>();
        app.init_resource::<CarbonTransportLimits>();
        app.init_resource::<CarbonTransportMetrics>();
        app.init_resource::<CarbonClientInbox>();

        app.register_message::<CarbonUpdateMessage>()
            .add_direction(NetworkDirection::ServerToClient);
        app.add_channel::<CarbonUpdateChannel>(ChannelSettings {
            mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
            ..Default::default()
        })
        .add_direction(NetworkDirection::ServerToClient);

        // Transform is a bounded f32 presentation projection. Child collider
        // descriptors and presentation impulse state are intentionally not
        // authoritative replication components.
        app.replicate::<DestinyBallId>();
        app.replicate::<DestinyBallMetadata>();
        app.replicate::<DestinyMass>();
        app.replicate::<DestinyPendingRemoval>();
        app.replicate::<Position>();
        app.replicate::<Rotation>();
        app.replicate::<LinearVelocity>();
        app.replicate::<AngularVelocity>();
        app.replicate::<Mass>();
        app.replicate::<MaxLinearSpeed>();
        app.replicate::<MaxAngularSpeed>();
        app.replicate::<RigidBody>();
        app.replicate::<Collider>();
        app.replicate::<ColliderDisabled>();
        app.replicate::<GravityScale>();

        app.add_systems(
            PostUpdate,
            (
                refresh_authenticated_bindings,
                sync_client_visibility_rooms,
                sync_ball_visibility_rooms,
                flush_carbon_outbox,
            )
                .chain()
                .before(MessageSystems::Send),
        );
        app.add_systems(
            PreUpdate,
            receive_carbon_frames.after(MessageSystems::Receive),
        );
    }
}

fn refresh_authenticated_bindings(
    clients: Query<(Entity, &CarbonClientIdentity)>,
    mut bindings: ResMut<CarbonClientBindings>,
    mut metrics: ResMut<CarbonTransportMetrics>,
) {
    bindings.by_id.clear();
    bindings.invalid_ids.clear();
    for (entity, identity) in &clients {
        if bindings.invalid_ids.contains(&identity.0) {
            continue;
        }
        if bindings.by_id.insert(identity.0, entity).is_some() {
            bindings.by_id.remove(&identity.0);
            bindings.invalid_ids.insert(identity.0);
            metrics.duplicate_bindings = metrics.duplicate_bindings.saturating_add(1);
        }
    }
}

fn ensure_global_room(
    rooms: &mut CarbonVisibilityRooms,
    allocator: &mut RoomAllocator,
) -> Option<RoomId> {
    if let Some(room) = rooms.global {
        return Some(room);
    }
    let room = allocate_visibility_room(rooms, allocator)?;
    rooms.global = Some(room);
    Some(room)
}

fn bubble_room(
    bubble_id: i64,
    rooms: &mut CarbonVisibilityRooms,
    allocator: &mut RoomAllocator,
) -> Option<RoomId> {
    if let Some(room) = rooms.bubbles.get(&bubble_id).copied() {
        return Some(room);
    }
    let room = allocate_visibility_room(rooms, allocator)?;
    rooms.bubbles.insert(bubble_id, room);
    Some(room)
}

fn personal_room(
    client_id: i64,
    rooms: &mut CarbonVisibilityRooms,
    allocator: &mut RoomAllocator,
) -> Option<RoomId> {
    if let Some(room) = rooms.personal.get(&client_id).copied() {
        return Some(room);
    }
    let room = allocate_visibility_room(rooms, allocator)?;
    rooms.personal.insert(client_id, room);
    Some(room)
}

fn allocate_visibility_room(
    rooms: &CarbonVisibilityRooms,
    allocator: &mut RoomAllocator,
) -> Option<RoomId> {
    let allocated = (if rooms.global.is_some() { 1_usize } else { 0_usize })
        .saturating_add(rooms.bubbles.len())
        .saturating_add(rooms.personal.len());
    if allocated >= MAX_VISIBILITY_ROOMS {
        return None;
    }
    // Lightyear 0.29 exposes only a panicking u16 allocator. Contain exhaustion
    // here and convert it into fail-closed visibility plus an observable metric.
    catch_unwind(AssertUnwindSafe(|| allocator.allocate())).ok()
}

fn sync_client_visibility_rooms(
    clients: Query<(
        Entity,
        &CarbonClientIdentity,
        &CarbonClientBubbles,
        Option<&Rooms>,
    )>,
    mut visibility: ResMut<CarbonVisibilityRooms>,
    mut allocator: ResMut<RoomAllocator>,
    mut metrics: ResMut<CarbonTransportMetrics>,
    mut commands: Commands,
) {
    let global = ensure_global_room(&mut visibility, &mut allocator);
    for (entity, identity, bubbles, current_rooms) in &clients {
        let mut replacement = Rooms::default();
        if let Some(global) = global {
            replacement.add_room(global);
        } else {
            metrics.room_allocation_failures =
                metrics.room_allocation_failures.saturating_add(1);
        }
        // Personal rooms are allocated lazily only when an owned cloaked ball
        // exists; ordinary authenticated-client churn must not consume the
        // finite Lightyear room-ID space.
        if let Some(personal) = visibility.personal.get(&identity.0).copied() {
            replacement.add_room(personal);
        }
        if bubbles.0.len() > MAX_BUBBLES_PER_CLIENT {
            // The host supplied an invalid interest set. Keep only the
            // fail-closed global/owner rooms already staged above and avoid an
            // attacker-controlled clone/sort on every PostUpdate.
            metrics.room_allocation_failures =
                metrics.room_allocation_failures.saturating_add(1);
            replace_rooms_if_changed(&mut commands, entity, current_rooms, replacement);
            continue;
        }
        let mut bubble_ids = bubbles.0.clone();
        bubble_ids.sort_unstable();
        bubble_ids.dedup();
        for bubble_id in bubble_ids.into_iter().filter(|value| *value >= 0) {
            if let Some(room) = bubble_room(bubble_id, &mut visibility, &mut allocator) {
                replacement.add_room(room);
            } else {
                metrics.room_allocation_failures =
                    metrics.room_allocation_failures.saturating_add(1);
            }
        }
        replace_rooms_if_changed(&mut commands, entity, current_rooms, replacement);
    }
}

fn sync_ball_visibility_rooms(
    balls: Query<(
        Entity,
        &DestinyBallMetadata,
        Option<&CarbonBallOwner>,
        Option<&DestinyPendingRemoval>,
        Option<&Rooms>,
    )>,
    bindings: Res<CarbonClientBindings>,
    mut visibility: ResMut<CarbonVisibilityRooms>,
    mut allocator: ResMut<RoomAllocator>,
    mut metrics: ResMut<CarbonTransportMetrics>,
    mut commands: Commands,
) {
    let global = ensure_global_room(&mut visibility, &mut allocator);
    for (entity, metadata, owner, pending, current_rooms) in &balls {
        let mut replacement = Rooms::default();
        if pending.is_some() {
            replace_rooms_if_changed(&mut commands, entity, current_rooms, replacement);
            continue;
        }
        if metadata.is_cloaked != 0 {
            if let Some(owner) = owner {
                if bindings.by_id.contains_key(&owner.0)
                    && !bindings.invalid_ids.contains(&owner.0)
                    && let Some(room) = personal_room(owner.0, &mut visibility, &mut allocator)
                {
                    replacement.add_room(room);
                } else if bindings.by_id.contains_key(&owner.0) {
                    metrics.room_allocation_failures =
                        metrics.room_allocation_failures.saturating_add(1);
                }
            }
        } else if metadata.is_global {
            if let Some(global) = global {
                replacement.add_room(global);
            } else {
                metrics.room_allocation_failures =
                    metrics.room_allocation_failures.saturating_add(1);
            }
        } else if metadata.new_bubble_id >= 0 {
            if let Some(room) = bubble_room(
                metadata.new_bubble_id,
                &mut visibility,
                &mut allocator,
            ) {
                replacement.add_room(room);
            } else {
                metrics.room_allocation_failures =
                    metrics.room_allocation_failures.saturating_add(1);
            }
        }
        replace_rooms_if_changed(&mut commands, entity, current_rooms, replacement);
    }
}

fn replace_rooms_if_changed(
    commands: &mut Commands,
    entity: Entity,
    current: Option<&Rooms>,
    replacement: Rooms,
) {
    let unchanged = current.is_some_and(|current| {
        current.rooms().count() == replacement.rooms().count()
            && replacement
                .rooms()
                .all(|room| current.contains_room(room))
    });
    if !unchanged {
        // Lightyear deliberately declares `Rooms` as an immutable Bevy
        // component so its relationship hooks cannot be bypassed through a
        // mutable query. Replacing the component runs those hooks and keeps
        // the allocator/visibility indices coherent.
        commands.entity(entity).insert(replacement);
    }
}

fn flush_carbon_outbox(
    mut outbox: ResMut<CarbonNetworkOutbox>,
    bindings: Res<CarbonClientBindings>,
    limits: Res<CarbonTransportLimits>,
    mut metrics: ResMut<CarbonTransportMetrics>,
    mut senders: Query<&mut MessageSender<CarbonUpdateMessage>>,
) {
    let mut sent_messages = 0usize;
    let mut sent_bytes = 0usize;
    loop {
        let Some((queue, envelope)) = next_envelope(&outbox) else {
            break;
        };
        let deliveries = match deliveries_from_envelope(envelope, *limits) {
            Ok(deliveries) => deliveries,
            Err(_) => {
                metrics.rejected_envelopes = metrics.rejected_envelopes.saturating_add(1);
                drop_front(&mut outbox, queue);
                continue;
            }
        };
        let delivery_count = deliveries.len();
        let mut frames = Vec::with_capacity(delivery_count);
        let mut unresolved = false;
        for (recipient_id, frame) in deliveries {
            let Some(entity) = bindings.by_id.get(&recipient_id).copied() else {
                metrics.unknown_recipients = metrics.unknown_recipients.saturating_add(1);
                unresolved = true;
                break;
            };
            if bindings.invalid_ids.contains(&recipient_id) || senders.get(entity).is_err() {
                metrics.unknown_recipients = metrics.unknown_recipients.saturating_add(1);
                unresolved = true;
                break;
            }
            frames.push((entity, frame));
        }
        frames.sort_by_key(|(_, frame)| frame.recipient_id);
        if unresolved {
            // Fail closed before the first side effect. Keeping an envelope
            // with a disconnected or unknown recipient at the front of this
            // globally ordered outbox would permanently block every later
            // batch, so reject it observably and require the product to rebase
            // that recipient after reconnect.
            metrics.rejected_envelopes = metrics.rejected_envelopes.saturating_add(1);
            drop_front(&mut outbox, queue);
            continue;
        }
        let frame_bytes = frames
            .iter()
            .map(|(_, frame)| encoded_len(frame))
            .try_fold(0usize, usize::checked_add);
        let Some(frame_bytes) = frame_bytes else {
            metrics.rejected_envelopes = metrics.rejected_envelopes.saturating_add(1);
            drop_front(&mut outbox, queue);
            continue;
        };
        // An envelope that cannot fit in an otherwise-empty flush would block
        // this globally ordered outbox forever. Treat that as malformed input,
        // discard it observably, and allow later batches to make progress.
        if frames.len() > limits.max_wire_messages_per_flush
            || frame_bytes > limits.max_wire_bytes_per_flush
        {
            metrics.rejected_envelopes = metrics.rejected_envelopes.saturating_add(1);
            metrics.backpressure_events = metrics.backpressure_events.saturating_add(1);
            drop_front(&mut outbox, queue);
            continue;
        }
        if sent_messages.saturating_add(frames.len()) > limits.max_wire_messages_per_flush
            || sent_bytes.saturating_add(frame_bytes) > limits.max_wire_bytes_per_flush
        {
            metrics.backpressure_events = metrics.backpressure_events.saturating_add(1);
            break;
        }
        let mut sender_disappeared = false;
        for (entity, frame) in frames {
            // The complete preflight above guarantees that every target has a
            // sender before the first side effect. Lightyear's `send` method
            // only appends to that per-link typed-message buffer and is
            // infallible; its later transport stage owns connection errors.
            let Ok(mut sender) = senders.get_mut(entity) else {
                sender_disappeared = true;
                break;
            };
            sender.send::<CarbonUpdateChannel>(frame);
        }
        if sender_disappeared {
            // Defensive retry path: an earlier frame may already be buffered,
            // so the recipient/batch dedup key makes a later retry idempotent.
            metrics.backpressure_events = metrics.backpressure_events.saturating_add(1);
            break;
        }
        sent_messages = sent_messages.saturating_add(delivery_count);
        sent_bytes = sent_bytes.saturating_add(frame_bytes);
        metrics.delivered_frames = metrics.delivered_frames.saturating_add(delivery_count as u64);
        metrics.delivered_bytes = metrics.delivered_bytes.saturating_add(frame_bytes as u64);
        drop_front(&mut outbox, queue);
    }
}

fn receive_carbon_frames(
    mut receivers: Query<(&CarbonClientIdentity, &mut MessageReceiver<CarbonUpdateMessage>)>,
    limits: Res<CarbonTransportLimits>,
    mut inbox: ResMut<CarbonClientInbox>,
    mut metrics: ResMut<CarbonTransportMetrics>,
) {
    for (identity, mut receiver) in &mut receivers {
        for frame in receiver.receive() {
            let bytes = encoded_len(&frame);
            if bytes == usize::MAX || bytes > limits.max_client_inbox_bytes {
                inbox.request_rebase();
                metrics.backpressure_events = metrics.backpressure_events.saturating_add(1);
                continue;
            }
            if frame.recipient_id != identity.0 {
                metrics.unauthorized_frames = metrics.unauthorized_frames.saturating_add(1);
                continue;
            }
            if frame.validate().is_err() {
                inbox.request_rebase();
                metrics.unauthorized_frames = metrics.unauthorized_frames.saturating_add(1);
                continue;
            }
            let dedup_key = (frame.recipient_id, frame.batch_id);
            if inbox.seen.contains(&dedup_key)
                || inbox
                    .last_batch_by_recipient
                    .get(&frame.recipient_id)
                    .is_some_and(|last| frame.batch_id <= *last)
            {
                metrics.duplicate_frames = metrics.duplicate_frames.saturating_add(1);
                continue;
            }
            if inbox.requires_rebase {
                continue;
            }
            if inbox.frames.len() >= limits.max_client_inbox_messages
                || inbox.queued_bytes.saturating_add(bytes) > limits.max_client_inbox_bytes
            {
                inbox.request_rebase();
                metrics.backpressure_events = metrics.backpressure_events.saturating_add(1);
                continue;
            }
            inbox.seen.insert(dedup_key);
            inbox.seen_order.push_back(dedup_key);
            inbox
                .last_batch_by_recipient
                .insert(frame.recipient_id, frame.batch_id);
            while inbox.seen_order.len() > DEDUP_WINDOW {
                if let Some(expired) = inbox.seen_order.pop_front() {
                    inbox.seen.remove(&expired);
                }
            }
            inbox.queued_bytes = inbox.queued_bytes.saturating_add(bytes);
            inbox.frames.push_back(frame);
        }
    }
}

#[derive(Clone, Copy)]
enum OutboxQueue {
    Singlecast,
    Narrowcast,
    Batch,
}

fn next_envelope(outbox: &CarbonNetworkOutbox) -> Option<(OutboxQueue, &Value)> {
    let candidates = [
        outbox.singlecasts.first().map(|value| (OutboxQueue::Singlecast, value)),
        outbox.narrowcasts.first().map(|value| (OutboxQueue::Narrowcast, value)),
        outbox.batches.first().map(|value| (OutboxQueue::Batch, value)),
    ];
    candidates.into_iter().flatten().min_by_key(|(_, value)| {
        value
            .get("batch_id")
            .and_then(Value::as_i64)
            .unwrap_or(i64::MAX)
    })
}

fn drop_front(outbox: &mut CarbonNetworkOutbox, queue: OutboxQueue) {
    let removed = match queue {
        OutboxQueue::Singlecast => (!outbox.singlecasts.is_empty())
            .then(|| outbox.singlecasts.remove(0)),
        OutboxQueue::Narrowcast => (!outbox.narrowcasts.is_empty())
            .then(|| outbox.narrowcasts.remove(0)),
        OutboxQueue::Batch => (!outbox.batches.is_empty()).then(|| outbox.batches.remove(0)),
    };
    if let Some(value) = removed {
        outbox.queued_bytes = outbox.queued_bytes.saturating_sub(encoded_len(&value));
    }
}

fn deliveries_from_envelope(
    envelope: &Value,
    limits: CarbonTransportLimits,
) -> Result<HashMap<i64, CarbonUpdateMessage>, String> {
    let object = envelope
        .as_object()
        .ok_or_else(|| "Carbon envelope must be an object".to_owned())?;
    let expected: HashSet<&str> =
        ["protocol", "schema_version", "mode", "batch_id", "updates"]
            .into_iter()
            .collect();
    if object.keys().map(String::as_str).collect::<HashSet<_>>() != expected
        || object.get("protocol").and_then(Value::as_str) != Some(CARBON_PROTOCOL_NAME)
        || object.get("schema_version").and_then(Value::as_u64)
            != Some(CARBON_SCHEMA_VERSION as u64)
    {
        return Err("invalid Carbon schema-v2 envelope".into());
    }
    let batch_id = object
        .get("batch_id")
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
        .ok_or_else(|| "invalid Carbon batch identifier".to_owned())?;
    let mode = object
        .get("mode")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing Carbon mode".to_owned())?;
    let encoded_updates = object.get("updates").cloned().unwrap_or(Value::Null);
    // Validate the complete tagged tree and its aggregate budgets, but route
    // the original representation. Decoding and then reserializing here used
    // to erase tuple tags from native deliveries.
    decode_canonical(encoded_updates.clone())?;
    let mut rows = Vec::new();
    let mut expanded = ExpandedDeliveryBudget::default();
    match mode {
        "singlecast" => rows.extend(rows_for_mode(encoded_updates, false, limits, &mut expanded)?),
        "narrowcast" => rows.extend(rows_for_mode(encoded_updates, true, limits, &mut expanded)?),
        "batch" => {
            let mut object = encoded_updates
                .as_object()
                .cloned()
                .ok_or_else(|| "Carbon batch updates must be an object".to_owned())?;
            if object.len() != 2
                || !object.contains_key("singlecasts")
                || !object.contains_key("narrowcasts")
            {
                return Err("Carbon batch has invalid fields".into());
            }
            rows.extend(rows_for_mode(
                object.remove("singlecasts").unwrap_or(Value::Null),
                false,
                limits,
                &mut expanded,
            )?);
            rows.extend(rows_for_mode(
                object.remove("narrowcasts").unwrap_or(Value::Null),
                true,
                limits,
                &mut expanded,
            )?);
        }
        _ => return Err(format!("invalid Carbon mode {mode:?}")),
    }
    let mut deliveries: HashMap<i64, CarbonUpdateMessage> = HashMap::new();
    for (recipient_id, row) in rows {
        if !deliveries.contains_key(&recipient_id)
            && deliveries.len() >= limits.max_wire_messages_per_flush
        {
            return Err("expanded Carbon recipient count exceeds the wire limit".into());
        }
        deliveries
            .entry(recipient_id)
            .or_insert_with(|| CarbonUpdateMessage {
                protocol: CARBON_PROTOCOL_NAME.into(),
                schema_version: CARBON_SCHEMA_VERSION,
                batch_id,
                recipient_id,
                updates: Vec::new(),
            })
            .updates
            .push(row);
    }
    Ok(deliveries)
}

#[derive(Default)]
struct ExpandedDeliveryBudget {
    rows: usize,
    bytes: usize,
}

fn rows_for_mode(
    value: Value,
    narrowcast: bool,
    limits: CarbonTransportLimits,
    budget: &mut ExpandedDeliveryBudget,
) -> Result<Vec<(i64, Value)>, String> {
    let rows = value
        .as_array()
        .ok_or_else(|| "Carbon updates must be an array".to_owned())?;
    let mut output = Vec::new();
    for encoded_row in rows {
        let decoded_row = decode_canonical(encoded_row.clone())?;
        let row = decoded_row
            .as_array()
            .ok_or_else(|| "Carbon update row must be an array or tuple".to_owned())?;
        if row.len() < 3 {
            return Err("Carbon update row is too short".into());
        }
        if !row
            .get(1)
            .and_then(Value::as_str)
            .is_some_and(|action| !action.is_empty())
            || !row.get(2).is_some_and(Value::is_array)
        {
            return Err("Carbon update row has an invalid action or state".into());
        }
        let recipients = if narrowcast {
            row[0]
                .as_array()
                .ok_or_else(|| "narrowcast recipients must be an array".to_owned())?
                .iter()
                .map(|value| {
                    value
                        .as_i64()
                        .ok_or_else(|| "recipient identifier must be int64".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![row[0]
                .as_i64()
                .ok_or_else(|| "singlecast recipient must be int64".to_owned())?]
        };
        let mut unique = recipients;
        unique.sort_unstable();
        unique.dedup();
        for recipient_id in unique {
            // Do not disclose a narrowcast's other recipient identifiers.
            let mut redacted = encoded_row.clone();
            replace_encoded_row_recipient(&mut redacted, recipient_id)?;
            budget.rows = budget
                .rows
                .checked_add(1)
                .ok_or_else(|| "expanded Carbon row count overflowed".to_owned())?;
            if budget.rows > MAX_EXPANDED_UPDATE_ROWS {
                return Err("expanded Carbon row count exceeds the compatibility limit".into());
            }
            budget.bytes = budget
                .bytes
                .checked_add(encoded_len(&redacted))
                .ok_or_else(|| "expanded Carbon byte count overflowed".to_owned())?;
            if budget.bytes > limits.max_wire_bytes_per_flush {
                return Err("expanded Carbon payload exceeds the wire byte limit".into());
            }
            output.push((recipient_id, redacted));
        }
    }
    Ok(output)
}

fn replace_encoded_row_recipient(row: &mut Value, recipient_id: i64) -> Result<(), String> {
    let values = match row {
        Value::Array(values) => values,
        Value::Object(object) if object.len() == 1 => object
            .get_mut("$destiny_bevy_tuple_v1")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| "Carbon update row must be an array or tuple".to_owned())?,
        _ => return Err("Carbon update row must be an array or tuple".into()),
    };
    let recipient = values
        .first_mut()
        .ok_or_else(|| "Carbon update row is too short".to_owned())?;
    *recipient = json!(recipient_id);
    Ok(())
}

fn encoded_len<T: Serialize>(value: &T) -> usize {
    struct Counter(usize);
    impl Write for Counter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0 = self
                .0
                .checked_add(buffer.len())
                .ok_or_else(|| io::Error::other("encoded length overflow"))?;
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, value).map_or(usize::MAX, |()| counter.0)
}

/// Replication markers start with no visibility. The sync system assigns a
/// global, bubble, or owner-personal room after authoritative metadata exists.
#[derive(Bundle)]
pub struct DestinyCarbonReplicationBundle {
    pub replicon: Replicated,
    pub lightyear: Replicate,
    pub rooms: Rooms,
}

impl DestinyCarbonReplicationBundle {
    pub fn hidden() -> Self {
        Self {
            replicon: Replicated,
            lightyear: Replicate::default(),
            rooms: Rooms::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(mode: &str, updates: Value) -> Value {
        json!({
            "protocol": CARBON_PROTOCOL_NAME,
            "schema_version": CARBON_SCHEMA_VERSION,
            "mode": mode,
            "batch_id": 17,
            "updates": updates,
        })
    }

    #[test]
    fn singlecast_delivery_is_recipient_specific() {
        let deliveries = deliveries_from_envelope(
            &envelope(
                "singlecast",
                json!([
                    {"$destiny_bevy_tuple_v1": [7, "DoDestinyUpdate", []]},
                    {"$destiny_bevy_tuple_v1": [9, "DoDestinyUpdate", []]},
                ]),
            ),
            CarbonTransportLimits::default(),
        )
        .expect("valid singlecast envelope");

        assert_eq!(
            deliveries.keys().copied().collect::<HashSet<_>>(),
            HashSet::from([7_i64, 9_i64]),
        );
        assert_eq!(deliveries[&7].recipient_id, 7);
        assert_eq!(deliveries[&7].updates.len(), 1);
        assert_eq!(deliveries[&9].recipient_id, 9);
        assert_eq!(deliveries[&9].updates.len(), 1);
    }

    #[test]
    fn narrowcast_redacts_other_recipient_identifiers() {
        let deliveries = deliveries_from_envelope(
            &envelope(
                "narrowcast",
                json!([
                    {"$destiny_bevy_tuple_v1": [
                        [7, 8],
                        "DoDestinyUpdate",
                        {"$destiny_bevy_tuple_v1": ["nested"]}
                    ]},
                ]),
            ),
            CarbonTransportLimits::default(),
        )
        .expect("valid narrowcast envelope");

        for recipient in [7, 8] {
            let decoded = decode_canonical(deliveries[&recipient].updates[0].clone())
                .expect("canonical row");
            let row = decoded
                .as_array()
                .expect("decoded row");
            assert_eq!(row[0], json!(recipient));
            assert!(deliveries[&recipient].updates[0]
                .get("$destiny_bevy_tuple_v1")
                .is_some());
            assert!(deliveries[&recipient].updates[0]["$destiny_bevy_tuple_v1"][2]
                .get("$destiny_bevy_tuple_v1")
                .is_some());
        }
    }

    #[test]
    fn malformed_or_unversioned_envelopes_fail_closed() {
        assert!(deliveries_from_envelope(
            &json!({
                "protocol": CARBON_PROTOCOL_NAME,
                "schema_version": 1,
                "mode": "singlecast",
                "batch_id": 17,
                "updates": [],
            }),
            CarbonTransportLimits::default(),
        )
        .is_err());
        assert!(deliveries_from_envelope(
            &envelope(
                "singlecast",
                json!([["not-an-int", "DoDestinyUpdate", []]]),
            ),
            CarbonTransportLimits::default(),
        )
        .is_err());
    }

    #[test]
    fn narrowcast_expansion_is_bounded_before_unlimited_cloning() {
        let mut limits = CarbonTransportLimits::default();
        limits.max_wire_bytes_per_flush = 16;
        assert!(deliveries_from_envelope(
            &envelope(
                "narrowcast",
                json!([
                    {"$destiny_bevy_tuple_v1": [[1, 2, 3], "DoDestinyUpdate", ["payload"]]},
                ]),
            ),
            limits,
        )
        .is_err());
    }

    #[test]
    fn inbound_delivery_rejects_cross_recipient_or_malformed_rows() {
        let valid = CarbonUpdateMessage {
            protocol: CARBON_PROTOCOL_NAME.into(),
            schema_version: CARBON_SCHEMA_VERSION,
            batch_id: 1,
            recipient_id: 7,
            updates: vec![json!([7, "DoDestinyUpdate", []])],
        };
        assert!(valid.validate().is_ok());

        let mut cross_recipient = valid.clone();
        cross_recipient.updates[0][0] = json!(8);
        assert!(cross_recipient.validate().is_err());

        let mut reserved_key_mixing = valid;
        reserved_key_mixing.updates[0][2] = json!({
            "$destiny_bevy_bytes_v1": "",
            "extra": true,
        });
        assert!(reserved_key_mixing.validate().is_err());
    }

    #[test]
    fn rebase_acknowledgement_requires_an_advancing_watermark() {
        let mut inbox = CarbonClientInbox::default();
        inbox.last_batch_by_recipient.insert(7, 41);
        inbox.request_rebase();

        assert!(inbox.acknowledge_rebase(7, 41).is_err());
        assert!(inbox.requires_rebase());
        inbox
            .acknowledge_rebase(7, 50)
            .expect("fresh full-state watermark");
        assert!(!inbox.requires_rebase());
        assert_eq!(inbox.last_batch_by_recipient.get(&7), Some(&50));
        assert!(inbox.seen.contains(&(7, 50)));
    }
}

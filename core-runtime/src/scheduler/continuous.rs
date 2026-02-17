//! Continuous batching for iteration-level dynamic batch membership.
//!
//! Requests join and leave the batch between token generation steps.

use std::collections::VecDeque;

use crate::engine::FinishReason;

/// Unique identifier for a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

/// Request phase in the continuous batch lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestPhase {
    /// Processing prompt tokens.
    Prefill,
    /// Generating output tokens.
    Decode,
    /// Request completed.
    Complete,
}

/// Slot in the continuous batch.
#[derive(Debug, Clone)]
pub struct BatchSlot {
    pub request_id: RequestId,
    pub phase: RequestPhase,
    pub tokens_generated: usize,
    pub max_tokens: usize,
    pub prompt_len: usize,
}

impl BatchSlot {
    /// Create a new slot for a request.
    pub fn new(request_id: RequestId, prompt_len: usize, max_tokens: usize) -> Self {
        Self {
            request_id,
            phase: RequestPhase::Prefill,
            tokens_generated: 0,
            max_tokens,
            prompt_len,
        }
    }

    /// Transition from prefill to decode phase.
    pub fn finish_prefill(&mut self) {
        self.phase = RequestPhase::Decode;
    }

    /// Record a generated token.
    pub fn record_token(&mut self) {
        self.tokens_generated += 1;
    }

    /// Check if generation is complete.
    pub fn is_complete(&self) -> bool {
        self.phase == RequestPhase::Complete
    }

    /// Mark as complete with reason.
    pub fn mark_complete(&mut self) {
        self.phase = RequestPhase::Complete;
    }
}

/// Result from a single step for one request.
#[derive(Debug, Clone)]
pub struct StepResult {
    pub request_id: RequestId,
    pub token: Option<u32>,
    pub finished: bool,
    pub finish_reason: Option<FinishReason>,
}

/// Pending request waiting to join the batch.
#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub request_id: RequestId,
    pub prompt_tokens: Vec<u32>,
    pub max_tokens: usize,
}

/// Continuous batcher with per-token iteration and dynamic membership.
#[derive(Debug)]
pub struct ContinuousBatcher {
    slots: Vec<Option<BatchSlot>>,
    _max_slots: usize,
    pending: VecDeque<PendingRequest>,
}

impl ContinuousBatcher {
    /// Create a new continuous batcher.
    pub fn new(max_slots: usize) -> Self {
        Self {
            slots: vec![None; max_slots],
            _max_slots: max_slots,
            pending: VecDeque::new(),
        }
    }

    /// Add a request to the pending queue.
    pub fn enqueue(&mut self, request: PendingRequest) {
        self.pending.push_back(request);
    }

    /// Admit pending requests into free slots.
    pub fn admit_pending(&mut self) -> Vec<(usize, PendingRequest)> {
        let mut admitted = Vec::new();
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                if let Some(req) = self.pending.pop_front() {
                    let batch_slot =
                        BatchSlot::new(req.request_id, req.prompt_tokens.len(), req.max_tokens);
                    *slot = Some(batch_slot);
                    admitted.push((idx, req));
                }
            }
        }
        admitted
    }

    /// Evict completed requests, freeing slots.
    pub fn evict_completed(&mut self) -> Vec<RequestId> {
        let mut evicted = Vec::new();
        for slot in &mut self.slots {
            if let Some(s) = slot {
                if s.is_complete() {
                    evicted.push(s.request_id);
                    *slot = None;
                }
            }
        }
        evicted
    }

    /// Get active slot count.
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Get pending request count.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get mutable reference to slot by index.
    pub fn get_slot_mut(&mut self, idx: usize) -> Option<&mut BatchSlot> {
        self.slots.get_mut(idx)?.as_mut()
    }

    /// Iterate over active slots.
    pub fn active_slots(&self) -> impl Iterator<Item = (usize, &BatchSlot)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|slot| (i, slot)))
    }

    /// Check if batch is empty (no active or pending requests).
    pub fn is_empty(&self) -> bool {
        self.active_count() == 0 && self.pending.is_empty()
    }
}

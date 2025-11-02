# Phase 1.3 - Event Stream & Frame Coalescing

**Version**: 1.4.0
**Duration**: 2-3 days
**Status**: 🔄 In Progress

---

## Objectives

Enhance the modern TUI with advanced event handling and rendering optimizations from Codex, while maintaining 100% visual compatibility with legacy.

### Goals

1. **Event Stream** - Switch from polling to event stream
2. **Frame Coalescing** - Batch rendering updates for efficiency
3. **Background Tasks** - Non-blocking operations
4. **Performance Metrics** - Track and log improvements

---

## Current State (Phase 1.2)

✅ **Completed:**
- Hybrid architecture (legacy + modern)
- 16ms event polling (vs 50ms legacy)
- Feature flag system
- Basic Ratatui integration
- Zero visual changes

🔴 **Known Issues:**
- Cursor positioning after command (deferred)

---

## Phase 1.3 Improvements

### 1. Event Stream (vs Polling)

**Current (Polling):**
```rust
// Poll every 16ms
if !event::poll(Duration::from_millis(16))? {
    return Ok(EventResult::Continue);
}
```

**Target (Event Stream):**
```rust
// Async event stream - no polling overhead
let mut events = EventStream::new();
while let Some(event) = events.next().await {
    // Process event immediately
}
```

**Benefits:**
- ✅ Zero CPU when idle (no polling loop)
- ✅ Instant response to events
- ✅ Better battery life on laptops
- ✅ More efficient for background tasks

---

### 2. Frame Coalescing

**Current:**
- Every event potentially triggers screen update
- Can cause flickering with rapid events

**Target:**
```rust
// Batch updates within 16ms window
let mut frame_scheduler = FrameScheduler::new(60); // 60 FPS
frame_scheduler.schedule_update(|| {
    // Render UI
});
```

**Benefits:**
- ✅ Smooth rendering (60 FPS cap)
- ✅ No flickering
- ✅ Better performance with rapid input
- ✅ Consistent frame timing

---

### 3. Background Task Handling

**Current:**
- Commands block until completion
- No concurrent operations

**Target:**
```rust
// Non-blocking command execution
let task = tokio::spawn(async move {
    process_command(&cmd).await
});

// UI remains responsive
// Show progress indicator
// Update when task completes
```

**Benefits:**
- ✅ Responsive UI during long operations
- ✅ Can cancel operations
- ✅ Progress feedback
- ✅ Multi-tasking support

---

### 4. Performance Metrics

**Current:**
- Basic logging only
- No runtime metrics

**Target:**
```rust
struct PerformanceMetrics {
    event_latency: Histogram,
    frame_time: Histogram,
    input_lag: Histogram,
}

// Log metrics periodically
metrics.log_summary();
```

**Benefits:**
- ✅ Track performance improvements
- ✅ Detect regressions
- ✅ User-visible stats (optional)
- ✅ Debugging aid

---

## Implementation Plan

### Step 1: Event Stream (Day 1)

**Files to modify:**
- `ui/src/modern/events.rs` - Switch to EventStream
- `cli/src/lib.rs` - Update event loop

**Changes:**
```rust
// events.rs
use crossterm::event::{EventStream, Event};
use futures::StreamExt;

pub async fn handle_events_stream(...) -> io::Result<EventResult> {
    let mut events = EventStream::new();

    match events.next().await {
        Some(Ok(Event::Key(key))) => handle_key_event(key, ...),
        Some(Ok(Event::Resize(w, h))) => handle_resize(w, h),
        _ => Ok(EventResult::Continue),
    }
}
```

**Testing:**
- Verify no CPU usage when idle
- Confirm instant response to keypress
- Check memory usage stable

---

### Step 2: Frame Scheduler (Day 2)

**Files to create:**
- `ui/src/modern/frame_scheduler.rs`

**Implementation:**
```rust
pub struct FrameScheduler {
    target_fps: u32,
    last_frame: Instant,
    pending_update: bool,
}

impl FrameScheduler {
    pub fn should_render(&mut self) -> bool {
        let elapsed = self.last_frame.elapsed();
        let frame_time = Duration::from_millis(1000 / self.target_fps as u64);

        if elapsed >= frame_time && self.pending_update {
            self.last_frame = Instant::now();
            self.pending_update = false;
            true
        } else {
            false
        }
    }

    pub fn request_update(&mut self) {
        self.pending_update = true;
    }
}
```

**Testing:**
- Rapid typing doesn't flicker
- Frame rate stays at 60 FPS
- CPU usage remains low

---

### Step 3: Background Tasks (Day 2-3)

**Files to modify:**
- `cli/src/lib.rs` - Add task spawning
- `commands/src/processor.rs` - Make async-friendly

**Implementation:**
```rust
// Spawn command in background
let (tx, rx) = tokio::sync::oneshot::channel();
tokio::spawn(async move {
    let result = process_command(&cmd).await;
    let _ = tx.send(result);
});

// UI loop remains responsive
loop {
    tokio::select! {
        event = handle_events(...) => { /* Handle input */ }
        result = rx => { /* Command completed */ }
    }
}
```

**Testing:**
- Long-running commands don't block input
- Can cancel operations
- UI stays responsive

---

### Step 4: Performance Metrics (Day 3)

**Files to create:**
- `ui/src/modern/metrics.rs`

**Implementation:**
```rust
pub struct UIMetrics {
    event_count: AtomicU64,
    frame_count: AtomicU64,
    avg_latency: AtomicU64,
}

impl UIMetrics {
    pub fn record_event(&self, latency: Duration) {
        self.event_count.fetch_add(1, Ordering::Relaxed);
        // Update running average
    }

    pub fn log_summary(&self) {
        info!(
            events = self.event_count.load(Ordering::Relaxed),
            frames = self.frame_count.load(Ordering::Relaxed),
            avg_latency_ms = self.avg_latency.load(Ordering::Relaxed),
            "UI Performance Summary"
        );
    }
}
```

**Testing:**
- Metrics accurate
- Low overhead
- Useful for debugging

---

## Success Criteria

### Performance
- ✅ Zero CPU when idle (event stream)
- ✅ Consistent 60 FPS rendering
- ✅ <16ms input latency maintained
- ✅ Non-blocking long operations

### Functionality
- ✅ All Phase 1.2 features preserved
- ✅ Visual appearance still identical to legacy
- ✅ No breaking changes
- ✅ Feature flag system maintained

### Quality
- ✅ No new bugs introduced
- ✅ Performance metrics tracked
- ✅ Documentation updated
- ✅ Tests passing

---

## Visual Compatibility

**CRITICAL**: Phase 1.3 improvements are **INTERNAL ONLY**.

**What changes:**
- Event handling mechanism (polling → stream)
- Frame scheduling (none → coalesced)
- Task execution (blocking → async)
- Metrics collection (none → tracked)

**What DOES NOT change:**
- ❌ Visual appearance
- ❌ User interface
- ❌ Command behavior
- ❌ Output formatting
- ❌ Terminal mode (still no alternate screen)

---

## Risk Assessment

### Low Risk ✅
- Event stream - well-tested pattern
- Frame scheduler - isolated component
- Metrics - read-only, no side effects

### Medium Risk ⚠️
- Background tasks - needs careful testing
- Cursor positioning bug still exists (deferred)

### Mitigation
- Feature flag system allows rollback
- Extensive testing before merge
- Legacy mode always available
- Incremental rollout possible

---

## Dependencies

### New Crates (if needed)
```toml
[dependencies]
futures = "0.3"  # For event stream
hdrhistogram = "7.5"  # For metrics (optional)
```

### Existing
- crossterm 0.28
- ratatui 0.28
- tokio 1.34

---

## Timeline

**Day 1:**
- Morning: Event stream implementation
- Afternoon: Testing and validation
- Evening: Documentation

**Day 2:**
- Morning: Frame scheduler
- Afternoon: Background task foundation
- Evening: Integration testing

**Day 3:**
- Morning: Performance metrics
- Afternoon: Full system testing
- Evening: Documentation and summary

---

## Rollback Plan

If Phase 1.3 introduces issues:

1. **Disable feature flag**
   ```bash
   cargo build  # Without modern-tui
   ```

2. **Revert commits**
   ```bash
   git revert <phase-1.3-commits>
   ```

3. **Stay on Phase 1.2**
   - 16ms polling still works
   - All basic features intact

---

## Next Phase Preview

### Phase 2 (2 weeks)
- Full async command pipeline
- Advanced terminal capabilities
- Smart caching
- Predictive input
- Enhanced error handling

---

## Current Status

**Phase**: 1.3
**Started**: 2024-11-02
**Target Completion**: 2024-11-04
**Current Step**: About to start Step 1 (Event Stream)

---

**Let's begin Phase 1.3! 🚀**

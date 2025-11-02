# NetToolsKit CLI - Feature Flags

This document describes the feature flag system for incremental TUI improvements.

## Overview

NetToolsKit CLI uses a feature flag system to enable opt-in TUI improvements without breaking existing functionality. This allows users to gradually adopt new features while maintaining stability.

## Available Features

### `legacy-ui` (default)
- **Status**: ✅ Stable
- **Description**: Current printf-style UI
- **Default**: Enabled by default
- **Use case**: Production use, stable workflows

### `modern-tui` (experimental)
- **Status**: 🚧 Experimental
- **Description**: New Ratatui-based TUI with rich widgets
- **Requires**: None
- **Enables**: Interactive widgets, better rendering, visual history

### `event-driven` (experimental)
- **Status**: 🚧 Experimental
- **Description**: Event-driven architecture instead of polling loop
- **Requires**: `modern-tui`
- **Enables**: Non-blocking UI, better performance, cancelable operations

### `frame-scheduler` (experimental)
- **Status**: 🚧 Experimental
- **Description**: Frame coalescing optimizer
- **Requires**: `modern-tui`
- **Enables**: Smooth rendering, reduced CPU usage, 60 FPS

### `persistent-sessions` (experimental)
- **Status**: 🚧 Experimental
- **Description**: Save and restore CLI sessions
- **Requires**: None
- **Enables**: Session history, resume from previous state

### `full-tui` (combo)
- **Status**: 🚧 Experimental
- **Description**: Enables all modern TUI features
- **Includes**: `modern-tui` + `event-driven` + `frame-scheduler`

### `experimental` (combo)
- **Status**: ⚠️ Highly Experimental
- **Description**: All experimental features enabled
- **Includes**: `full-tui` + `persistent-sessions`

## Usage

### Compile-Time Features

Enable features when building:

```bash
# Default build (legacy UI)
cargo build --release

# Modern TUI only
cargo build --release --features modern-tui

# Full modern TUI
cargo build --release --features full-tui

# All experimental features
cargo build --release --features experimental
```

### Runtime Environment Variables

Override features at runtime (highest priority):

```bash
# Enable modern TUI
NTK_USE_MODERN_TUI=1 ntk

# Enable modern TUI + event-driven
NTK_USE_MODERN_TUI=1 NTK_USE_EVENT_DRIVEN=1 ntk

# Try full modern stack
NTK_USE_MODERN_TUI=1 NTK_USE_EVENT_DRIVEN=1 NTK_USE_FRAME_SCHEDULER=1 ntk
```

Environment variables accept these values as "enabled":
- `1`, `true`, `yes`, `on` (case-insensitive)

### Permanent Configuration

Set environment variables permanently:

```bash
# Bash/Zsh
echo 'export NTK_USE_MODERN_TUI=1' >> ~/.bashrc

# PowerShell
[System.Environment]::SetEnvironmentVariable('NTK_USE_MODERN_TUI', '1', 'User')

# Windows CMD
setx NTK_USE_MODERN_TUI 1
```

## Feature Detection

Check which features are enabled:

```bash
# Run with verbose logging
ntk --verbose
```

Or programmatically:

```rust
use nettoolskit_core::Features;

let features = Features::detect();
println!("Modern TUI: {}", features.use_modern_tui);
features.print_status();
```

## Migration Guide

### Phase 1: Try Modern TUI
```bash
# Test without commitment
NTK_USE_MODERN_TUI=1 ntk

# If you like it, enable permanently
export NTK_USE_MODERN_TUI=1
```

### Phase 2: Enable Event-Driven
```bash
# Adds non-blocking architecture
export NTK_USE_EVENT_DRIVEN=1
```

### Phase 3: Enable Frame Scheduler
```bash
# Adds smooth 60 FPS rendering
export NTK_USE_FRAME_SCHEDULER=1
```

### Phase 4: Full Modern Stack
```bash
# All modern features
export NTK_USE_MODERN_TUI=1
export NTK_USE_EVENT_DRIVEN=1
export NTK_USE_FRAME_SCHEDULER=1
```

## Rollback

Disable any feature by unsetting the environment variable:

```bash
# Bash/Zsh
unset NTK_USE_MODERN_TUI

# PowerShell
[System.Environment]::SetEnvironmentVariable('NTK_USE_MODERN_TUI', $null, 'User')

# Or just don't set it
ntk  # Uses default (legacy UI)
```

## Compatibility

### What Changes
- ✅ UI/UX improvements
- ✅ Performance optimizations
- ✅ New interactive features

### What Doesn't Change
- ✅ All CLI commands work identically
- ✅ Command-line arguments unchanged
- ✅ Configuration files compatible
- ✅ Templates work the same
- ✅ Exit codes unchanged

## Testing

Run regression tests to ensure features don't break existing functionality:

```bash
# Test default features
cargo test

# Test with modern TUI
cargo test --features modern-tui

# Test all features
cargo test --all-features
```

## Troubleshooting

### Modern TUI doesn't work
```bash
# Check if feature is actually enabled
cargo build --features modern-tui

# Or at runtime
NTK_USE_MODERN_TUI=1 ntk --verbose
```

### Want to go back to legacy UI
```bash
# Just unset the variable
unset NTK_USE_MODERN_TUI

# Or run without it
ntk
```

### Performance issues
```bash
# Try enabling frame scheduler
NTK_USE_FRAME_SCHEDULER=1 ntk
```

## Roadmap

| Version | Features | Status |
|---------|----------|--------|
| v1.0.0 | Legacy UI | ✅ Released |
| v1.1.0 | Feature flags, Modern TUI basic | 🚧 In Progress |
| v1.2.0 | Event-driven architecture | 📅 Planned |
| v1.3.0 | Interactive widgets, Sessions | 📅 Planned |
| v1.4.0 | Modern TUI default | 📅 Planned |
| v2.0.0 | Remove legacy UI | 📅 Future |

## Contributing

When adding new features:

1. Add feature flag to `core/Cargo.toml`
2. Document in this README
3. Add regression tests
4. Update migration guide
5. Test with all feature combinations

## Support

- **Stable**: Use `legacy-ui` (default)
- **Experimental**: Use at your own risk
- **Issues**: Report on GitHub with feature flags used

## License

Same as NetToolsKit CLI (MIT)
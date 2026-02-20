# AGENTS.md - src/app/

iced 0.14 GUI module. Orchestrates state, messages, and rendering.

## Structure

```
app/
├── mod.rs          # WeakAuraImporter state, update(), view()
├── message.rs      # Message enum (97 variants)
├── state.rs        # Sub-state structs
├── actions/        # Business logic (impl WeakAuraImporter)
│   ├── handlers.rs # Message dispatch
│   ├── import.rs   # Import flow + conflict handling
│   ├── loading.rs  # Aura parsing (async)
│   └── removal.rs  # Aura deletion
└── ui/             # Rendering (impl WeakAuraImporter)
    ├── main_panel.rs # Content area
    ├── sidebar.rs    # Aura tree
    └── dialogs.rs    # Modals (conflict, confirm, setup)
```

## Patterns

### Modular Impl (Distributed Methods)
Methods on `WeakAuraImporter` are split across files:
- `actions/*.rs` — State mutations, async tasks
- `ui/*.rs` — `render_*()` methods returning `Element<Message>`

```rust
// In actions/import.rs
impl WeakAuraImporter {
    pub fn start_import(&mut self) -> Command<Message> { ... }
}

// In ui/sidebar.rs
impl WeakAuraImporter {
    pub fn render_sidebar(&self) -> Element<Message> { ... }
}
```

### State Composition
Main state composed of focused sub-structs (defined in `state.rs`):

| Struct | Purpose |
|--------|---------|
| `UiVisibility` | Dialog/panel toggle flags |
| `SidebarState` | Width, resize, expanded groups |
| `TaskProgress` | Loading/importing/removing status |
| `ConflictState` | Conflict detection results, resolutions |
| `RemovalState` | Selected auras for removal |
| `SavedVariablesState` | WoW path, loaded file, aura tree |
| `StatusState` | Status bar message, last result |

### Message Flow
1. User action → `Message` variant
2. `update()` matches and delegates to `actions/handlers.rs`
3. Handler mutates state, optionally returns `Command<Message>`
4. `view()` calls `render_*()` methods from `ui/*.rs`

### Async Pattern
Long operations use channels + progress updates:

```rust
// Start task, return Command
Command::perform(
    async_operation(),
    |result| Message::LoadingUpdate(result)
)

// Progress variants
LoadingUpdate::Progress { current, total, message }
LoadingUpdate::Complete { entries, added, duplicates, errors }
LoadingUpdate::Error(String)
```

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Add new message | `message.rs` enum, `handlers.rs` match |
| Change import flow | `actions/import.rs` |
| Modify dialog UI | `ui/dialogs.rs` |
| Update sidebar tree | `ui/sidebar.rs` |
| Add state field | `state.rs` struct, `mod.rs` WeakAuraImporter |

## Anti-Patterns

- **Direct state access in ui/**: Always pass through `&self`, never `&mut self`
- **Blocking in update()**: Use `Command::perform()` for async ops
- **Inline styles**: Use `theme::*` functions from root `theme.rs`
- **Raw widget creation**: Wrap buttons in consistent spacing (see `main_panel.rs` line 324)

## Theme Integration

All styling via `crate::theme`:
```rust
use crate::theme::{self, colors, spacing, typography};

container(content)
    .style(theme::container_panel)
    .padding(spacing::MD)

button(text("Import").size(typography::BODY))
    .style(theme::button_primary)
```

Color palette: `BG_*` (backgrounds), `GOLD*` (accents), `TEXT_*`, `SUCCESS/ERROR/WARNING`

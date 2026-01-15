# Skill Web UI - Production Ready PRD

## Overview

The Skill Engine Web UI (skill-web) currently has 37+ non-functional UI elements including buttons, forms, dropdowns, and checkboxes that either have no handlers or incomplete implementations. This PRD outlines the complete plan to make the UI fully production-ready.

## Current State

- **Framework:** Yew 0.21 (Rust WASM)
- **State Management:** Yewdux 0.11
- **Styling:** TailwindCSS
- **API Client:** gloo-net

### Audit Results Summary

| Category | Count | Description |
|----------|-------|-------------|
| Buttons without handlers | 18 | Stub buttons with no onclick |
| Dropdowns without handlers | 6 | Select elements missing onchange |
| Form inputs without handlers | 4 | Text/number inputs not bound |
| Checkboxes without handlers | 5 | Toggle controls not functional |
| Partial implementations | 3 | Started but incomplete |
| Hardcoded mock data | 1 | History page uses fake data |
| **Total** | **37+** | Non-functional elements |

---

## Phase 1: Critical Path - Core Functionality

### 1.1 Skill Installation Flow

**Priority: P0**

**Current State:** "Install Skill" buttons exist but do nothing.

**Requirements:**
- Create `InstallSkillModal` component with:
  - Source type selector (URL, Git, Local, Registry)
  - Source input field with validation
  - Instance name (optional)
  - Force re-clone toggle for Git sources
  - Installation progress indicator
  - Error handling with user-friendly messages
- Wire up API call to `POST /api/skills`
- After success: refresh skills list, show success notification
- Handle errors: display error modal with retry option

**Files to modify:**
- `src/pages/skills.rs` - Add modal state and button handlers
- `src/components/mod.rs` - Export new component
- NEW: `src/components/install_skill_modal.rs`

**API Endpoint:** `POST /api/skills`
```json
{
  "source": "github:user/repo@v1.0.0",
  "instance": "production",
  "force": false
}
```

---

### 1.2 Skill Uninstall Flow

**Priority: P0**

**Current State:** "Uninstall" button on skill detail page does nothing.

**Requirements:**
- Create `ConfirmationModal` reusable component
- Show confirmation with skill name and warning
- Wire up API call to `DELETE /api/skills/{name}`
- After success: navigate back to skills list, show notification
- Handle errors gracefully

**Files to modify:**
- `src/pages/skill_detail.rs` - Add uninstall handler
- NEW: `src/components/confirmation_modal.rs`

**API Endpoint:** `DELETE /api/skills/{name}`

---

### 1.3 Instance Management (Create/Edit/Delete)

**Priority: P0**

**Current State:**
- Instance editor modal opens but save doesn't call API
- Delete button on instances does nothing

**Requirements:**

**Create Instance:**
- Wire `InstanceEditorModal` save callback to API
- Call `POST /api/skills/{name}/instances`
- Refresh skill detail page after success

**Edit Instance:**
- Call `PUT /api/skills/{name}/instances/{instance}`
- Refresh skill detail page after success

**Delete Instance:**
- Add confirmation modal before delete
- Call `DELETE /api/skills/{name}/instances/{instance}`
- Refresh skill detail page after success

**Files to modify:**
- `src/pages/skill_detail.rs` - Wire up all instance operations
- `src/components/instance_editor.rs` - Connect save to API

**API Endpoints:**
- `POST /api/skills/{name}/instances`
- `PUT /api/skills/{name}/instances/{instance}`
- `DELETE /api/skills/{name}/instances/{instance}`

---

### 1.4 Skill Configuration

**Priority: P0**

**Current State:** "Configure" button on skill detail page does nothing.

**Requirements:**
- Open instance editor for default instance when clicked
- Or show instance selector if multiple instances exist
- Allow editing configuration values
- Persist to API

**Files to modify:**
- `src/pages/skill_detail.rs` - Add configure button handler

---

## Phase 2: History & Execution

### 2.1 History Page - Real Data Loading

**Priority: P0**

**Current State:** Uses hardcoded mock data (lines 29-75 in history.rs).

**Requirements:**
- Load execution history from `GET /api/executions`
- Implement pagination (Previous/Next buttons)
- Show loading state while fetching
- Handle empty state
- Handle error state

**Files to modify:**
- `src/pages/history.rs` - Replace mock data with API calls

**API Endpoint:** `GET /api/executions?page=1&per_page=20`

---

### 2.2 History Page - Filters

**Priority: P1**

**Current State:** Filter dropdowns exist but don't work.

**Requirements:**
- Skill filter: Filter by skill name
- Status filter: Filter by execution status (success/error/running)
- Time range filter: Last hour, 24h, 7d, 30d, all
- Filters should trigger API reload with query params
- Save filter preferences to localStorage

**Files to modify:**
- `src/pages/history.rs` - Add filter state and handlers

**API Endpoint:** `GET /api/executions?skill=aws&status=success&since=24h`

---

### 2.3 Execution Detail View

**Priority: P1**

**Current State:** "View" button on execution rows does nothing.

**Requirements:**
- Create `ExecutionDetailModal` component
- Show: skill, tool, parameters, output, timing, status
- Format output based on content type (JSON/text)
- Copy output button
- Re-run button (pre-fills run page)

**Files to modify:**
- `src/pages/history.rs` - Add view button handler
- NEW: `src/components/execution_detail_modal.rs`

---

### 2.4 Re-run Execution

**Priority: P2**

**Current State:** "Re-run" button does nothing.

**Requirements:**
- Navigate to Run page with pre-filled:
  - Skill name
  - Tool name
  - Previous parameters
- Allow user to modify before executing

**Files to modify:**
- `src/pages/history.rs` - Add re-run handler
- `src/pages/run.rs` - Accept URL params for pre-fill

---

### 2.5 History Refresh

**Priority: P2**

**Current State:** "Refresh" button does nothing.

**Requirements:**
- Reload execution history from API
- Show loading indicator during refresh
- Maintain current filters/pagination

**Files to modify:**
- `src/pages/history.rs` - Add refresh handler

---

## Phase 3: Settings Page

### 3.1 Settings State Management

**Priority: P1**

**Current State:** All settings inputs are hardcoded with no state.

**Requirements:**
- Create proper settings state in Yewdux store
- Load settings from API on page mount
- Track dirty state (unsaved changes)
- Warn before leaving with unsaved changes

**Files to modify:**
- `src/store/settings.rs` - Implement actual store usage
- `src/pages/settings.rs` - Connect to store

**API Endpoint:** `GET /api/config`, `PUT /api/config`

---

### 3.2 Execution Settings

**Priority: P1**

**Files to modify:** `src/pages/settings.rs`

**Elements to wire up:**
- Default Timeout input (line 72-78)
  - Bind to state, add oninput handler
  - Validate: 1-300 seconds
- Metadata checkbox (line 117)
  - Add onchange handler
  - Persist to config

---

### 3.3 Search Settings

**Priority: P1**

**Elements to wire up:**
- Embedding Provider dropdown (line 132-136)
  - Options: FastEmbed, OpenAI, Ollama
  - Persist selection
- Vector Store dropdown (line 146-149)
  - Options: In-Memory, LanceDB, Qdrant
  - Persist selection
- Hybrid Search checkbox (line 153)
  - Toggle BM25 + Vector search
- Reranking checkbox (line 160)
  - Toggle cross-encoder reranking

---

### 3.4 Data Management

**Priority: P2**

**Elements to wire up:**
- History Retention input (line 176-182)
  - Validate: 100-10000
  - Persist to config
- Clear History button (line 188)
  - Confirmation modal
  - Call `DELETE /api/executions`
- Export Data button (line 189)
  - Download JSON file with all data
- Import Data button (line 190)
  - File picker
  - Parse and import JSON

---

### 3.5 Application Actions

**Priority: P2**

**Elements to wire up:**
- Check for Updates button (line 212)
  - Call version check API
  - Show update available modal if newer version
- Reset to Defaults button (line 218)
  - Confirmation modal
  - Reset all settings
- Save Changes button (line 219)
  - Call `PUT /api/config`
  - Show success/error notification

---

## Phase 4: Search & Navigation

### 4.1 Global Search (Navbar)

**Priority: P1**

**Current State:** Search input captures query but doesn't display results.

**Requirements:**
- Debounced search (300ms delay)
- Search dropdown showing results
- Categories: Skills, Tools, Commands
- Keyboard navigation (arrow keys, enter)
- Press `/` to focus search
- Press `Escape` to close
- Click outside to close

**Files to modify:**
- `src/components/navbar.rs` - Implement search dropdown
- NEW: `src/components/search_results.rs`

**API Endpoint:** `POST /api/search`

---

### 4.2 Command Palette

**Priority: P2**

**Requirements:**
- Open with `Cmd/Ctrl + K`
- Quick actions: Install skill, Run tool, Go to page
- Recent commands
- Fuzzy search

**Files to modify:**
- NEW: `src/components/command_palette.rs`
- `src/app.rs` - Add keyboard listener

---

## Phase 5: Onboarding Flow

### 5.1 Credentials Step

**Priority: P1**

**Current State:** API key inputs don't capture values.

**Requirements:**
- Bind inputs to state
- Validate API key format
- Test API key with provider
- Show validation status (valid/invalid)
- Store securely (not in localStorage)

**Files to modify:**
- `src/pages/onboarding.rs` - Wire up credential inputs

---

### 5.2 Skills Selection Step

**Priority: P1**

**Current State:** Checkboxes are static.

**Requirements:**
- Track selected skills in state
- Install selected skills during "Finish" step
- Show installation progress
- Handle partial failures

**Files to modify:**
- `src/pages/onboarding.rs` - Wire up skill checkboxes

---

## Phase 6: Notifications & Error Handling

### 6.1 Toast Notifications

**Priority: P1**

**Requirements:**
- Create global notification system
- Types: success, error, warning, info
- Auto-dismiss after 5 seconds
- Manual dismiss button
- Stack multiple notifications
- Position: top-right

**Files to modify:**
- `src/store/ui.rs` - Already has types, need to implement
- NEW: `src/components/notifications.rs`
- `src/app.rs` - Render notification container

---

### 6.2 Error Boundaries

**Priority: P2**

**Requirements:**
- Catch component errors
- Show error UI instead of crashing
- Log errors for debugging
- Retry button

---

## Phase 7: Polish & UX

### 7.1 Loading States

**Priority: P1**

**Requirements:**
- Skeleton loaders for lists
- Spinner for buttons during action
- Progress bar for long operations
- Disable buttons during loading

---

### 7.2 Empty States

**Priority: P2**

**Requirements:**
- Meaningful empty state for each page
- Call-to-action button
- Illustration or icon

---

### 7.3 Keyboard Shortcuts

**Priority: P2**

| Shortcut | Action |
|----------|--------|
| `/` | Focus search |
| `Cmd+K` | Command palette |
| `Escape` | Close modal/dropdown |
| `?` | Show shortcuts help |

---

## Technical Implementation Notes

### State Management Pattern

```rust
// In component
let (store, dispatch) = use_store::<SettingsStore>();

// Read state
let timeout = store.default_timeout;

// Update state
dispatch.apply(SettingsAction::SetTimeout(60));
```

### API Call Pattern

```rust
let api = Api::new();
let on_click = {
    let api = api.clone();
    Callback::from(move |_| {
        let api = api.clone();
        spawn_local(async move {
            match api.skills.uninstall("skill-name").await {
                Ok(_) => {
                    // Show success notification
                    // Refresh data
                }
                Err(e) => {
                    // Show error notification
                }
            }
        });
    })
};
```

### Form Input Pattern

```rust
let value = use_state(|| String::new());

let on_input = {
    let value = value.clone();
    Callback::from(move |e: InputEvent| {
        let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
        value.set(input.value());
    })
};

html! {
    <input
        type="text"
        value={(*value).clone()}
        oninput={on_input}
    />
}
```

---

## API Endpoints Required

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/skills` | List skills |
| POST | `/api/skills` | Install skill |
| GET | `/api/skills/{name}` | Get skill detail |
| DELETE | `/api/skills/{name}` | Uninstall skill |
| POST | `/api/skills/{name}/instances` | Create instance |
| PUT | `/api/skills/{name}/instances/{inst}` | Update instance |
| DELETE | `/api/skills/{name}/instances/{inst}` | Delete instance |
| GET | `/api/executions` | List executions |
| GET | `/api/executions/{id}` | Get execution detail |
| DELETE | `/api/executions` | Clear history |
| POST | `/api/execute` | Execute tool |
| POST | `/api/search` | Semantic search |
| GET | `/api/config` | Get config |
| PUT | `/api/config` | Update config |
| GET | `/api/health` | Health check |
| GET | `/api/version` | Version info |

---

## Definition of Done

Each task is complete when:
1. UI element is fully functional
2. API integration works
3. Loading state is shown
4. Errors are handled gracefully
5. Success feedback is provided
6. Component compiles without warnings
7. Manual testing passes

---

## Timeline Estimate

| Phase | Scope | Estimated Tasks |
|-------|-------|-----------------|
| Phase 1 | Core Functionality | 8 tasks |
| Phase 2 | History & Execution | 10 tasks |
| Phase 3 | Settings Page | 8 tasks |
| Phase 4 | Search & Navigation | 4 tasks |
| Phase 5 | Onboarding Flow | 4 tasks |
| Phase 6 | Notifications | 3 tasks |
| Phase 7 | Polish & UX | 5 tasks |
| **Total** | | **42 tasks** |

---

## Success Metrics

- [ ] 0 non-functional buttons
- [ ] 0 hardcoded mock data
- [ ] 100% of forms submit correctly
- [ ] All API endpoints connected
- [ ] Error states handled for all operations
- [ ] Loading states for all async operations
- [ ] Notifications for all user actions

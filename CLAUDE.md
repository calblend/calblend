# Calblend Project Context

## Overview
Calblend is a cross-platform calendar integration library providing a unified API for Google Calendar, Outlook, and native calendars. Core engine in Rust, exposed to Node.js via napi-rs FFI bindings, with TypeScript SDK wrapper.

## Key Documents
- **Project Specification**: `docs/PROJECT_SPEC.md` - Full feature requirements
- **Implementation Roadmap**: `docs/IMPLEMENTATION_ROADMAP.md` - Phasing and gaps
- **Unified Calendar Design**: `docs/unified-calendar-design.md` - Data model details

## Architecture Invariants
- **NO MICROSERVICES**: Compiles to native Node.js addon (.node file)
- **ZERO RUNTIME DEPS**: No external services, databases, or daemons
- **PLUGGABLE STORAGE**: Users provide token persistence via trait/interface
- **TYPE SAFETY END-TO-END**: Rust → FFI → TypeScript with no `any` types

## Project Structure
```
calblend/
├── crates/
│   ├── calblend-core/     # Rust calendar engine
│   └── calblend-ffi/       # napi-rs FFI bindings
├── packages/
│   └── node/               # TypeScript SDK (@calblend/calendar)
├── examples/               # Example applications
└── docs/                   # Documentation
```

## Key Commands
```bash
# Build everything
cargo build --workspace && npm run build -w @calblend/calendar

# Test with FFI validation
cargo test --workspace && npm test -w @calblend/calendar

# Generate TypeScript definitions from Rust
npm run generate-types -w @calblend/calendar

# Cross-compile for production
npm run build:release -w @calblend/calendar
```

## FFI Patterns & Anti-Patterns

### ✅ CORRECT: Safe FFI Error Handling
```rust
// In calblend-ffi/src/lib.rs
#[napi]
impl CalendarClient {
    #[napi]
    pub async fn list_events(&self, calendar_id: String) -> Result<Vec<Event>> {
        self.inner
            .list_events(&calendar_id)
            .await
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}
```

### ❌ AVOID: Exposing Raw Pointers
```rust
// NEVER do this - memory safety violation
#[napi]
pub fn get_calendar_ptr() -> *mut Calendar { ... }
```

### ✅ CORRECT: Type-Safe Data Transfer
```rust
// Define serializable types for FFI boundary
#[napi(object)]
#[derive(Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub start: DateTime,
    pub end: DateTime,
}
```

### ❌ AVOID: Complex Lifetime Management
```rust
// Don't expose Rust lifetimes through FFI
pub struct EventRef<'a> { ... } // This won't work with napi-rs
```

## Token Management Pattern
```typescript
// User provides storage implementation
interface TokenStorage {
  getToken(provider: CalendarProvider): Promise<TokenData | null>;
  saveToken(provider: CalendarProvider, token: TokenData): Promise<void>;
  removeToken(provider: CalendarProvider): Promise<void>;
}

// Library uses it internally
const client = new CalendarClient({
  tokenStorage: new MyDatabaseTokenStorage(),
  providers: ['google', 'outlook']
});
```

## Testing Strategy
1. **Unit tests** in Rust for core logic
2. **Integration tests** in TypeScript for FFI boundary
3. **Memory leak detection** with Valgrind/ASAN
4. **Cross-platform CI** for Windows/macOS/Linux

## Performance Considerations
- Batch API calls to minimize FFI overhead
- Use async/await throughout (Rust tokio → Node.js event loop)
- Profile FFI boundary with `node --prof`
- Target <10ms for calendar queries

## Error Handling Hierarchy
1. Rust errors → Converted to napi::Error
2. napi::Error → Becomes JavaScript Error
3. JavaScript Error → Wrapped in CalblendError class
4. CalblendError → Provides error codes and recovery hints

## Build & Distribution
- Pre-built binaries via GitHub Actions
- Falls back to local compilation
- Supports Node.js 20.11.0+ 
- Uses @napi-rs/cli for cross-compilation

## Code Style
- Rust: `cargo fmt` with default settings
- TypeScript: Prettier with 2-space indentation
- Commit format: `type(scope): message` (e.g., `feat(ffi): add event filtering`)

## Common Workflows
- Adding new calendar provider: Implement trait in core, expose in FFI, wrap in SDK
- Debugging FFI issues: Enable `NAPI_RS_LOG=debug` environment variable
- Memory profiling: Use `node --expose-gc` with heap snapshots

## Architecture Decision Records
- **Why Rust?** Performance + memory safety for calendar sync operations
- **Why napi-rs?** Best FFI ergonomics, async support, TypeScript generation
- **Why pluggable storage?** Flexibility for serverless, Electron, mobile contexts
- **Why unified API?** Reduce complexity for consumers, handle quirks internally
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Strudel is a live coding pattern language for making music on the web, inspired by TidalCycles. It's a JavaScript/TypeScript port of the Haskell-based TidalCycles pattern language, designed for real-time algorithmic music composition in the browser.

## Development Commands

### Setup
```bash
pnpm i                    # Install dependencies (required before development)
```

### Development
```bash
pnpm dev                  # Start the REPL dev server (runs jsdoc-json prebuild step)
pnpm start                # Alias for pnpm dev
```

### Testing
```bash
pnpm test                 # Run all tests (runs jsdoc-json prebuild step)
pnpm test-ui              # Run tests with UI
pnpm test-coverage        # Run tests with coverage report
pnpm snapshot             # Regenerate test snapshots (when updating/creating pattern functions)
vitest run <path>         # Run single test file
```

### Code Quality
```bash
pnpm lint                 # Check for lint errors
pnpm codeformat           # Format all files with prettier
pnpm format-check         # Check if files are well formatted
pnpm check                # Run format-check + lint + test (full CI check)
```

### Building
```bash
pnpm build                # Build the website (runs jsdoc-json prebuild step)
pnpm preview              # Preview production build
pnpm tauri build          # Build standalone desktop version
```

### Other
```bash
pnpm jsdoc-json           # Generate documentation JSON (auto-runs before test/build/dev)
pnpm osc                  # Start OSC server
pnpm sampler              # Start sample server
```

### Environment Variables

The following environment variables can be used to configure the development environment:

**WebSocket Configuration:**
- `STRUDEL_WS_PORT` - WebSocket server port for external editor integration (default: 8080)

Example usage:
```bash
STRUDEL_WS_PORT=9090 pnpm dev   # Run dev server with WebSocket on custom port
```

This is useful when:
- Running multiple Strudel instances simultaneously
- The default port 8080 is already in use
- Working with external editors that require a specific port configuration

## Architecture

### Monorepo Structure

This is a pnpm workspace monorepo with packages in `/packages/` and the main website in `/website/`. The workspace setup allows packages to reference each other using `workspace:*` dependencies.

**Key architectural layers:**

1. **Core Pattern Engine** (`@strudel/core`): The heart of Strudel
   - `pattern.mjs`: Core `Pattern` class with Haskell-style functor/applicative/monadic operations
   - `hap.mjs`: `Hap` (happening) represents a pattern event with a time span and value
   - `timespan.mjs`: Time representation using fractions for precise timing
   - `state.mjs`: Query state for pattern evaluation
   - `controls.mjs`: Large collection of pattern transformation functions (effects, filters, etc.)
   - Patterns are functions that map time `State` to arrays of `Hap`s

2. **Mini Notation Parser** (`@strudel/mini`):
   - Parses the mini notation DSL (e.g., `"bd sd [hh hh]"`)
   - Built with PEG parser (`krill.pegjs`) - run `pnpm build:parser` if modifying grammar
   - Integrates with core via string parser registration

3. **Transpiler** (`@strudel/transpiler`):
   - Converts user-written JavaScript into evaluatable Strudel code
   - Uses `acorn` for parsing and `escodegen` for code generation
   - Handles syntactic sugar and pattern shortcuts

4. **Audio Output** (`@strudel/webaudio`):
   - Web Audio API integration
   - Depends on `superdough` and `supradough` for audio synthesis
   - Audio worklet bundling via custom Vite plugin

5. **REPL/Website** (`website/`):
   - Built with Astro (static site generator) + React components
   - CodeMirror integration (`@strudel/codemirror`) for the code editor
   - Supports multiple keybinding modes: CodeMirror (default), Vim, Emacs, Helix, and VSCode
   - Main entry point: `website/src/pages/`
   - REPL components: `website/src/repl/`

6. **Extension Packages**: Many optional packages for additional functionality:
   - `@strudel/tonal`: Musical/tonal helpers
   - `@strudel/midi`: MIDI support
   - `@strudel/osc`: OSC (Open Sound Control) support
   - `@strudel/hydra`: Hydra visuals integration
   - `@strudel/csound`: Csound integration
   - `@strudel/draw`: Pattern visualization
   - And many more...

### Important Patterns

- **JSDoc generation is critical**: The `jsdoc-json` command generates `doc.json` which is used by the documentation system and REPL autocomplete. It runs automatically before `test`, `build`, and `dev`.

- **Fraction-based timing**: All time values use the `fraction.js` library for mathematical precision, avoiding floating-point errors in rhythm calculation.

- **Pattern composition**: Patterns are immutable and composed functionally. Most pattern methods return new patterns rather than mutating.

- **Licensing**: AGPL-3.0-or-later - all code must be shareable under the same license.

### Testing

- Tests use Vitest
- Test files are in each package's `test/` directory
- Configuration: `vitest.config.mjs` at root
- Snapshots are used extensively for pattern testing - regenerate with `pnpm snapshot`
- Tests run in non-isolated mode (`isolate: false`) for performance

### Publishing Workflow

Packages are published to npm independently with Lerna version management:
1. `npx lerna version --no-private` - bump versions
2. `pnpm --filter "./packages/**" publish --dry-run` - preview
3. `pnpm --filter "./packages/**" publish --access public` - publish

**CRITICAL**: Always publish with `pnpm`, never `npm` (npm doesn't support `publishConfig` overrides used in all packages).

### Version Tag System

Strudel uses a versioning system to prevent breaking changes from affecting old patterns stored in the database. Patterns can include `// @version x.y` metadata to lock behavior to a specific version. See CONTRIBUTING.md lines 165-237 for the database patching workflow.

## Key Files to Know

- `/packages/core/pattern.mjs` - Core pattern implementation (~3000+ lines)
- `/packages/core/controls.mjs` - Pattern control functions (~2000+ lines)
- `/packages/mini/krill.pegjs` - Mini notation grammar (PEG)
- `/packages/transpiler/transpiler.mjs` - Code transpilation logic
- `/website/astro.config.mjs` - Website build configuration
- `/doc.json` - Generated API documentation (auto-generated, don't edit)
- `/vitest.config.mjs` - Test configuration

## Common Development Scenarios

### Adding a New Pattern Function

1. Add the function to the appropriate package (likely `@strudel/core/controls.mjs`)
2. Write JSDoc comments with `@example` tags
3. Add tests in the package's `test/` directory
4. Run `pnpm snapshot` if using snapshot tests
5. The function will automatically appear in documentation and autocomplete via jsdoc-json

### Modifying the Mini Notation Parser

1. Edit `/packages/mini/krill.pegjs`
2. Run `cd packages/mini && pnpm build:parser` to regenerate the parser
3. Add tests for new syntax
4. Update documentation if adding user-facing features

### Working on the Website/REPL

1. Run `pnpm dev` from root (not from website directory)
2. Website served at http://localhost:4321 by default
3. Changes to package code require restart (not hot-reloaded)
4. React components in `website/src/components/`
5. REPL-specific code in `website/src/repl/`

### Adding New Keybindings

The REPL supports multiple keybinding modes through CodeMirror extensions. To add a new keybinding mode:

1. **Add the package dependency** to `/packages/codemirror/package.json`
2. **Import the keybindings** in `/packages/codemirror/keybindings.mjs`
3. **Register in the keymaps object** in `keybindings.mjs`
4. **Add UI option** in `/website/src/repl/components/panel/SettingsTab.jsx` - add to the ButtonGroup items
5. **Create documentation** in `/website/src/pages/technical-manual/[keymap-name].mdx` (optional but recommended)

Example: Helix keybindings were added using the `codemirror-helix` package (v0.5.0)

### Custom Keybinding Commands

Some keybinding modes have Strudel-specific commands that dispatch custom events:
- **`repl-evaluate`**: Evaluate the current code (Vim `:w`, Helix `Space w`)
- **`repl-stop`**: Stop/pause playback (Vim `:q`, Helix `Space q`)
- **`repl-toggle-comment`**: Toggle line comments (Vim `gc`, Helix `Space c`)

To add custom commands to a keybinding mode, create a keymap extension with `keymap.of()` and combine it with the base keybindings using an extension function. See `helixExtension` and `helixStrudelCommands` in `keybindings.mjs` for an example.

# Cleanup Plan: Consolidate to fastn + malai terminology

## Overview
Consolidating the project structure to use only `fastn` (framework) and `malai` (CLI tool) terminology, removing `kulfi` to simplify communication and organization.

## Goal
- Move `kulfi-id52` → `fastn-id52` in fastn/v0.5
- Move `kulfi-utils` → `fastn-net` in fastn/v0.5  
- Rename `fastn-stack/kulfi` repo → `fastn-stack/malai`
- Remove all kulfi terminology from active use

## Phase 1: Prepare fastn-id52 crate

### 1. Create fastn-id52 in fastn/v0.5
- Copy kulfi-id52 code to fastn/v0.5/fastn-id52
- Update Cargo.toml: name = "fastn-id52"
- Update all internal documentation references
- Ensure it's a clean, standalone crate with no kulfi references

### 2. Publish fastn-id52 to crates.io
- Version 0.1.0
- Update README with proper documentation
- Add migration note for users of kulfi-id52

## Phase 2: Prepare fastn-net crate

### 3. Create fastn-net in fastn/v0.5
- Copy kulfi-utils code to fastn/v0.5/fastn-net
- Update Cargo.toml: name = "fastn-net"
- Replace dependency: kulfi-id52 → fastn-id52
- Update all imports and references
- Remove any kulfi-specific branding/naming

### 4. Update fastn-net internals
- Rename any remaining kulfi references in code
- Update documentation
- Ensure clean API surface

## Phase 3: Rename kulfi repo to malai

### 5. Update GitHub repository
- Rename fastn-stack/kulfi → fastn-stack/malai
- GitHub will create redirects automatically

### 6. Update malai crate in the renamed repo
- Already exists, just needs cleanup
- Update dependencies: 
  - kulfi-utils → fastn-net
  - kulfi-id52 → fastn-id52
- Remove kulfi crate entirely (or rename to malai-daemon if needed)

## Phase 4: Update fastn/v0.5 to use new crates

### 7. Update fastn dependencies
- Add fastn-net as dependency
- Add fastn-id52 as dependency
- Remove any direct iroh dependencies (use through fastn-net)

## Phase 5: Cleanup and deprecation

### 8. Mark old crates as deprecated
- Publish final versions of kulfi-utils and kulfi-id52 with deprecation notices
- Point users to new crates

### 9. Update documentation
- Update all READMEs
- Update any blog posts or documentation
- Update installation scripts

## Detailed TODO List

### Week 1: Crate Migration
- [ ] Fork kulfi-id52 → fastn/v0.5/fastn-id52
  - [ ] Update Cargo.toml package name
  - [ ] Update all docs to remove kulfi references
  - [ ] Ensure all tests pass
  - [ ] Publish fastn-id52 v0.1.0 to crates.io

- [ ] Fork kulfi-utils → fastn/v0.5/fastn-net
  - [ ] Update Cargo.toml package name
  - [ ] Change dependency: kulfi-id52 → fastn-id52
  - [ ] Update all imports in code
  - [ ] Rename modules if needed (e.g., kulfi-specific names)
  - [ ] Ensure all tests pass
  - [ ] Publish fastn-net v0.1.0 to crates.io

### Week 2: Repository Restructuring
- [ ] Rename GitHub repo: fastn-stack/kulfi → fastn-stack/malai
- [ ] In malai repo:
  - [ ] Update malai/Cargo.toml dependencies:
    - [ ] kulfi-utils → fastn-net
    - [ ] kulfi-id52 → fastn-id52
  - [ ] Update all imports in malai code
  - [ ] Delete or archive kulfi crate directory
  - [ ] Update workspace Cargo.toml
  - [ ] Update CI/CD workflows
  - [ ] Update README

### Week 3: Integration
- [ ] Update fastn/v0.5:
  - [ ] Add fastn-net dependency
  - [ ] Add fastn-id52 dependency
  - [ ] Integrate networking capabilities
  - [ ] Update fastn documentation

- [ ] Testing:
  - [ ] Test malai with new dependencies
  - [ ] Test fastn with fastn-net integration
  - [ ] Ensure peer-to-peer communication works

### Week 4: Cleanup
- [ ] Deprecate old crates on crates.io:
  - [ ] Publish kulfi-utils with deprecation notice
  - [ ] Publish kulfi-id52 with deprecation notice
  
- [ ] Update documentation:
  - [ ] malai README
  - [ ] fastn documentation
  - [ ] Installation guides
  - [ ] Blog post about the change (optional)

- [ ] Update external references:
  - [ ] malai.sh website
  - [ ] Any tutorials or guides

## Key Benefits

1. **Cleaner naming**: Just `fastn` (framework) and `malai` (CLI tool)
2. **Better organization**: Network code in fastn repo where it's needed
3. **Easier communication**: No need to explain what kulfi is
4. **Logical separation**: 
   - fastn = web framework with p2p capabilities
   - malai = standalone p2p networking tool

## Migration Path for Users

### For Rust Dependencies
```toml
# Old (deprecated)
[dependencies]
kulfi-utils = "0.1"
kulfi-id52 = "0.1"

# New
[dependencies] 
fastn-net = "0.1"
fastn-id52 = "0.1"
```

### For CLI Users
```bash
# Old
kulfi start
kulfi identity create

# New (malai handles the daemon/identity)
malai start
malai identity create
```

## Notes

- GitHub's automatic redirects will handle repository rename
- Crates.io deprecation notices will guide users to new packages
- All existing functionality preserved, just reorganized
- This consolidation makes the project easier to understand and communicate
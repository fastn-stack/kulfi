# malai Development Plan

## High-Level Implementation Strategy

### Phase 1: Core Daemon Foundation
Build basic daemon with MALAI_HOME management

### Phase 2: P2P Communication Layer  
Implement fastn-p2p protocols for config distribution and SSH execution

### Phase 3: Service Mesh
Add TCP/HTTP forwarding with identity injection

### Phase 4: Production Ready
Add security hardening, error handling, performance optimization

---

## Detailed Implementation Plan

### Phase 1: Core Daemon Foundation

#### **1.1 Basic Daemon Process Management**
- [ ] Implement lockfile creation and checking
- [ ] Implement auto-daemonization (fork to background)
- [ ] Implement --foreground mode
- [ ] Test: daemon starts, creates lockfile, prevents multiple instances

#### **1.2 MALAI_HOME Structure Management** 
- [ ] Fix directory paths (remove ssh/ subdirectory)
- [ ] Implement cluster directory scanning
- [ ] Implement role detection (cluster-manager/machine/client)
- [ ] Test: malai daemon scans and reports correct roles

#### **1.3 Basic CLI Communication**
- [ ] Implement Unix socket server in daemon
- [ ] Implement socket client in CLI commands  
- [ ] Simple JSON protocol for CLI ↔ daemon
- [ ] Test: malai info talks to daemon via socket

### Phase 2: P2P Communication Layer

#### **2.1 Config Distribution (Cluster Manager)**
- [ ] Implement config change monitoring
- [ ] Implement state.json management  
- [ ] Implement P2P config sending via fastn-p2p
- [ ] Test: config changes distribute to machines

#### **2.2 Config Reception (Machines)**
- [ ] Implement P2P config listener
- [ ] Implement machine-config.toml writing
- [ ] Implement config validation and sender verification
- [ ] Test: machines receive and apply configs

#### **2.3 SSH Execution (Basic)**
- [ ] Implement P2P SSH request protocol
- [ ] Implement SSH daemon listener  
- [ ] Implement command execution with permission checking
- [ ] Test: malai web01.company ps aux works end-to-end

### Phase 3: Service Mesh

#### **3.1 TCP Service Forwarding**
- [ ] Parse services.toml configuration
- [ ] Implement TCP port listeners
- [ ] Implement P2P TCP forwarding
- [ ] Test: mysql -h localhost:3306 works

#### **3.2 HTTP Service Forwarding**
- [ ] Implement HTTP server on port 80
- [ ] Implement subdomain routing (admin.company.localhost)
- [ ] Implement client ID52 header injection
- [ ] Test: http://admin.company.localhost works

#### **3.3 Service Management Commands**
- [ ] Implement malai service add/remove/list
- [ ] Implement services.toml editing
- [ ] Implement dynamic service reloading
- [ ] Test: service management workflow

### Phase 4: Production Ready

#### **4.1 Security Hardening**
- [ ] Input validation and sanitization
- [ ] Username validation for privilege escalation prevention
- [ ] Group loop detection in permission resolution
- [ ] Command injection prevention

#### **4.2 Error Handling & Reliability**
- [ ] Graceful P2P connection failure handling
- [ ] Automatic reconnection logic
- [ ] Service health checking
- [ ] Comprehensive logging

#### **4.3 Performance Optimization**
- [ ] Connection pooling optimization
- [ ] Service discovery caching
- [ ] Config distribution batching
- [ ] Memory usage optimization

---

## Current Status
- ✅ **Design Complete**: Architecture documented, CLI structure working
- ✅ **Basic Commands**: cluster init, machine init, info working
- ✅ **Framework Ready**: MALAI_HOME scanning, role detection foundation
- 🚧 **Next**: Phase 1.1 - Basic daemon process management

## Development Approach
1. **Small increments**: Each task is independently testable
2. **Test-driven**: Write test, implement to pass, commit
3. **Working software**: Keep basic functionality working at each step
4. **Proper todos**: Use todo!() for unimplemented parts, no simulation code

Each phase builds on the previous one while maintaining working functionality.
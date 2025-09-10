# malai MVP Release Plan

## 🎯 MVP Features (Release Blockers)

### **✅ IMPLEMENTED (Ready)**
1. **P2P Infrastructure**: ConfigUpdate + ExecuteCommand protocols working end-to-end
2. **Role Detection**: cluster.toml vs machine.toml with configuration error handling
3. **File Structure**: Design-compliant clusters/ directory with identity per cluster
4. **Config Validation**: `malai rescan --check` comprehensive TOML validation
5. **Basic Command Execution**: Real remote command execution via P2P
6. **E2E Testing**: Comprehensive business logic testing with proper file structure

### **❌ NOT IMPLEMENTED (MVP Blockers)**
1. **Real malai daemon**: Single daemon with multi-identity P2P listeners
2. **Multi-cluster daemon startup**: One daemon handles all cluster identities simultaneously
3. **Config management**: Remote config download/upload/edit commands with hash validation
4. **Command aliases**: Global aliases in malai.toml (`malai web` → `malai web01.company ps aux`)
5. **Basic ACL system**: Group expansion and permission validation (simple implementation)

## 🚀 Post-MVP Features (Next Releases)

### **Release 2: Service Mesh**
1. **TCP forwarding**: `mysql -h localhost:3306` → remote MySQL via P2P
2. **HTTP forwarding**: `curl admin.company.localhost` → remote admin interface

### **Release 3: Advanced Features**  
1. **CLI → daemon socket communication**: Connection pooling optimization
2. **Self-command optimization**: Cluster manager bypass P2P for self-operations
3. **Advanced ACL**: Complex group hierarchies and command-specific permissions
4. **Identity management**: Rich identity commands replacing keygen

## 📋 MVP Implementation Plan

### **Phase 1: Real Daemon (Critical)**
```rust
// Replace broken core_utils.rs daemon with clean implementation
async fn run_real_malai_daemon() {
    let cluster_roles = scan_cluster_roles().await?;
    
    for (cluster_alias, identity, role) in cluster_roles {
        spawn malai_server(identity, role);  // One listener per identity
    }
    
    // Handle all clusters in single daemon process
}
```

### **Phase 2: Config Management (Critical)**
```bash
# Must work for MVP
malai config download company           # Download cluster.toml with hash
malai config upload company.toml        # Upload with hash validation  
malai config edit company               # Atomic edit with $EDITOR
```

### **Phase 3: Command Aliases (Critical)**
```toml
# malai.toml
[aliases]
web = "web01.company ps aux"
db = "db01.company backup"
logs = "web01.company tail -f /var/log/nginx/access.log"
```

### **Phase 4: Basic ACL (Critical)**
```rust
// Simple but functional ACL
- Wildcard permissions: allow_from = "*"
- Direct ID matching: allow_from = "client-id52"
- Basic group support: allow_from = "admins" (simple group expansion)
```

## 🎯 MVP Success Criteria

**User can:**
1. **Setup cluster**: `malai cluster init company` → working cluster manager
2. **Join machines**: Machine gets config via P2P, accepts commands  
3. **Execute commands**: `malai web01.company ps aux` works remotely
4. **Manage config**: Download, edit, upload cluster config remotely
5. **Use aliases**: `malai web` executes predefined commands
6. **Multi-cluster**: Same device participates in multiple clusters

**Technical requirements:**
- Single `malai daemon` handles all clusters and identities
- Real P2P communication between devices (proven working)
- Basic security with ACL validation  
- Clean, maintainable code organization

## 🚫 Explicitly NOT in MVP
- Connection pooling via CLI sockets (optimization for later)
- Service mesh (TCP/HTTP forwarding)  
- Advanced ACL features (complex group hierarchies)
- Self-command optimization (efficiency improvement)
- Advanced config features (three-way merge, etc.)

This MVP provides **complete distributed infrastructure functionality** with all essential features for real-world usage.
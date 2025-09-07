# `malai ssh`

## Clusters

`malai ssh` lets you create clusters. Each cluster contains a cluster manager,
which is a server that has `malai ssh` running. Any server or any machine in 
malai cluster can belong to one or more clusters. Each cluster is uniquely
identified by the id52 of cluster manager, but each cluster can give itself a
cluster alias as well. Cluster aliases are based on domain, so aliases can be
unique. Cluster manager's ID52 can be stored in DNS as text record, so clusters
can be identified either the id52 of cluster manager or the domain name, which 
contains a txt record containing the same cluster manager id52.

## Servers vs Devices

`malai ssh` comes with multiple sub commands, `malai ssh server`, this is what
every server that wants to be connectable by other machines in cluster will run.

Devices that do not expose services, can not allow incoming `malai ssh` 
connections, so cluster members can either be servers or devices. Servers will
accept `malai ssh` commands (and optionally one or more http service access).

## Server and Device Aliases and IDs

Each server or device has to create their id52. In the cluster manager toml file,
each device/server to be added is listed. In the config file we define the alias
for each id52. And we define which services, can be contacted by which aliases,
so the cluster config file acts like a global alias and ACL file.

So each machine in cluster has an alias, say `foo`, and if it belongs to 
`amitu.com` cluster then full name of `foo` would be `foo.amitu.com`. If domains
are not available, then `<foo-id52>.<cluster-id52>` also works. In fact since
machines are identified by alias defined in config, we will also have 
`foo.<cluster-id52>` that will work.

Each service exposed by a server will also have unique alias on that server, so
say if we have exposed HTTP:8080 on `foo.amitu.com` by alias `django-admin`, we
get full name of service as `django-admin.foo.amitu.com` or 
`django-admin.<foo-id52>.<cluster-id52>` or `django-admin.foo.<cluster-id52>`.

## HTTP Proxy

On any server in the cluster we can have any number of http services running,
and we can expose them by first giving them an alias, and then configuring ACL
to decide which devices can connect with this service.

## Config File Format

```toml
[cluster-manager]
id52 = ""

use-keyring = true # default, if false either of following required

private-key-file = "" # optional
private-key = "" # optional

[server.foo]
id52 = ""
# any server listed here as full access
allow-from = "comma separated <id52>"  

[server.foo.ls]
# these only can run ls command
allow-from = "comma separated <id52>"

[server.foo.service.django]
http = 8000
allow-from = "comma separated <id52>"

[device.ios]
id52 = ""

[group.my-servers]
members = "list of aliases comma separated"
```

## Config File Syncing

The cluster manager is aware of all known devices in the cluster, and can contact
each of those devices via p2p, so it will keep a hash of cluster config against
each device that they have seen last. It will have a poller which will see if
toml file hash is diff from what we know for any given device, if so it will try
to send the latest config file to device.

### Sans Private Key 

Of course when sharing it will not send `use-keyring`, `private-key-file` or
`private-key`. We may choose to share even lesser data based on what's really 
essential for that device/server (like each server needs to know who can connect
with services that server exposes, so that server can reject disallowed 
connections). 

## `malai ssh <server-id> <optional-cmd>`

This is the command people will use to run a command on the server. Based on
the cluster id part of server-id, this command will figure out which 
`<client-id52>` should used to connect to which server. 

Meaning we support possibility of this device to be part of multiple clusters,
for each cluster it will have a unique id52 keypair. 

All the keys etc., will be stored in `DATADIR[malai]/ssh/clusters/<cluster-alias>`.

### Content of `<cluster-alias>` folder

`cluster-manager.toml`, this is

## `malai ssh agent`

`malai ssh` can run with or without `malai ssh agent` running. If the agent is
running, it runs as a process in background, and `malai ssh` learns about this
background process via environment variable, `MALAI_SSH_AGENT`, which will 
contain its unix socket. If MALAI_SSH_AGENT is not set or down, it will fall
back to making fresh connection for every invocation. If agent is available
`malai ssh` will send the command to `malai ssh agent`, which will maintain
connections in process.

Since we can not log things on stdout/stderr, which has to be reserved for the
command that is run over `malai ssh`, logs will be stored in `LOGDIR[malai]/ssh`
folder.

## Lockdown Mode

One can enable lockdown mode by using env variable: `MALAI_LOCKDOWN_MODE=true`. 
In this mode `malai ssh` will never try to read secret keys (as it will just not 
be available to anyone other than `malai ssh agent`), and `malai ssh agent` 
would be required. 

## HTTP Proxy

We want `curl django-admin.foo.amitu.com` to work, for this to happen `malai ssh 
agent` HAS to be running. This will work because `HTTP_PROXY` environment 
variable will point to HTTP proxy that `malai ssh agent` is running, which 
figures out 

## `malai ssh curl`

Will always use `malai ssh agent` HTTP PROXY by calling curl by auto-setting the
`HTTP_PROXY` environment variable, this ensures we always are using the right
command to connect to our private services.

## `malai ssh agent --envirnoment` (`-e` for short)

`eval "$(malai ssh agent --envirnoment [--lockdown] [--http])"` can be
used to start (if needed) and print the environment variables
`MALAI_LOCKDOWN_MODE` (if `--lockdown` is passed, which is true by default),
`MALAI_SSH_AGENT` and `HTTP_PROXY` (if `--http` is passed, which is default,
to switch it off say `--http=false`).

## Single Agent For All Clusters

The `malai ssh agent` will pick all clusters in `DATADIR[malai]/ssh/clusters/`.

This means a single HTTP proxy, single background agent, etc., for all clusters.

# Audit, signing, and monitoring

Audit events are emitted by the Envctl processes into Meta-owned durable state and
are signed according to the vault policy. Yazelix captures each managed process's
stdout and stderr beneath `/home/flexnetos/meta/var/lib/yazelix/runtime/logs` and
records exact process identities beneath its service runtime root.

Monitoring must validate the process identity, authenticated readiness, vault
unlock plus USB possession, and canonical database paths. A listening port alone is
not proof of ownership. Adding a sink or monitor requires a pinned Yazelix binary,
an absolute owned path, a readiness assertion, and a flake regression check.

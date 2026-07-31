# Cognitum USB ceremony

Provisioning creates the USB-bound vault factor and pinned Cognitum trust material.
Normal operation is automatic: plug in the USB and start Yazelix. The Yazelix stack
bootstrap discovers the mounted trust material, establishes the owned link-local
connection, starts the authenticated stores and `secretd`, calls the hard USB unlock,
and starts dependents only after the vault reports unlocked with USB possession.

The reconnect monitor repeats that owned lifecycle after removal and reinsertion.
A missing USB is a fail-closed startup condition for the complete stack, not a reason
to start databases, agents, or the runner under a competing path.

# CI and supply-chain contract

Every runtime binary is pinned and built by the Yazelix flake. Auto-update changes
the upstream release pin and its integrity metadata, then rebuilds the same profile;
it must not download an unowned binary at runtime.

CI must prove that installed binaries and launch scripts contain no retired home
agent roots, no volatile login-session roots, and no service-manager ownership. It
must also exercise the startup ordering and negative cases: wrong process identity,
missing USB possession, locked vault, duplicate runner, unauthenticated database,
and a legacy Envctl service-unit manifest field.

# Envctl component contract

Envctl components install or reconcile artifacts; they do not create a parallel
runtime supervisor. The `Wiring` schema permits owned path, shell, desktop, package,
Nix, CDI, alternatives, data, and config artifacts. Unknown fields fail parsing.

Long-running secrets processes are composed only by Yazelix's packaged stack
bootstrap. New runtime dependencies are added to the Yazelix flake, passed to the
bootstrap as absolute build-time substitutions, ordered behind their readiness
checks, and covered by flake checks. New Rust crates remain Envctl workspace members;
new Meta peers remain independent repositories registered in `.meta.yaml`.

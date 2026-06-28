# Standard nushell config. Loaded by login nushell (`nu -l`, `nu -l -c`) and
# interactive non-yazelix nu. NOT loaded by bare `nu -c` (nushell loads no
# config in that mode) and NOT by yazelix sessions (they use an explicit
# --config; see ~/.config/yazelix/shell_nu.nu which sources the same module).
# Relative source: nushell resolves `source` against the directory of this
# file, so the sibling module loads regardless of $HOME (portability: no
# hardcoded path). Was an absolute /home/drdave path before ADR-0006 wave 2.
source rtk-wrappers.nu
# Meta /usr mirror on PATH/LD_LIBRARY_PATH (relative source: resolved against this
# file's dir, so it loads regardless of $HOME). See the module header for rationale.
source meta-usr-path.nu

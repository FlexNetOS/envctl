# Standard host Nushell config. Yazelix sessions use their packaged managed
# config and native `rtk_wrappers.nu`; this host layer must not duplicate those
# command definitions.
# Standalone login Nu loads that same module from the stable profile-owned
# runtime tree, so profile activation updates both paths without copying defs.
use ~/.nix-profile/nushell/config/rtk_wrappers.nu *
# Meta /usr mirror on PATH/LD_LIBRARY_PATH (relative source: resolved against this
# file's dir, so it loads regardless of $HOME). See the module header for rationale.
source meta-usr-path.nu

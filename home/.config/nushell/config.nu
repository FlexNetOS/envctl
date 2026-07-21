# Standard host Nushell config. Yazelix sessions use their packaged managed
# config and native `rtk_wrappers.nu`; this host layer must not duplicate those
# command definitions.
# Standalone login Nu loads that same module from the stable profile-owned
# runtime tree, so profile activation updates both paths without copying defs.
use ~/.nix-profile/nushell/config/rtk_wrappers.nu *
# Strict profile-only PATH normalization. The relative source resolves against
# this file's directory and therefore follows the installed config atomically.
source profile-path.nu

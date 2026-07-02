# comment rows are ignored
source ./shared.nu
const META_USR_BIN_SUBDIRS = ["usr/bin" "usr/sbin"]
let root = ($env.META_ROOT? | default "/tmp/flex")
$env.PATH = ($META_USR_BIN_SUBDIRS | append $env.PATH | uniq)
$env.SECRET_TOKEN = "fixture-token"
def --wrapped git [...rest] { ^rtk git ...$rest }
if ("META_ROOT" in $env) {
    $env.LD_LIBRARY_PATH = "lib"
}
use std/log

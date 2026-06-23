# TASK-0076 guardian report — PASS (runtime-proven)
Invariants: no-c PASS (no dep change), one-rustls N/A, engine-single-lib N/A (manifest-only),
fail-closed PASS (worker exits 0 on absent Seed/daemon; the HOME-unbound regression that broke
this was caught by runtime-verify and fixed), shape PASS, no-system-depth = matches sibling
/usr/local+/etc convention (udev irreducibly system; family relocation is separate).
Gates: no-c shape enable p7 kdf-feature-off agent-env loop-state harness-scripts = ALL PASS.
Lock: envctl.lock 79 components, lock --check rc=0, [components.cognitum-seed-autounlock] present.
Runtime (Phase 3.5, hardware-in-the-loop, Seed present at /run/media/drdave/COGNITUM):
  - component discovered (auto-detect), detect=FALSE on clean box (honest predicate)
  - after install: oneshot ExecStart code=exited status=0/SUCCESS
  - journal: "autounlock: vault unlocked via USB possession factor"
  - secretctl status: unlocked  usb_possessed=true  → setpriv owner-drop SO_PEERCRED fix WORKS
Deferred (honest): a true reboot-persistence test (boot with Seed plugged, no session) needs
linger + a physical reboot — owner hardware-in-the-loop. The hotplug/owner-session path is proven.
VERDICT: PASS-WITH-NOTES (feature runtime-proven; reboot-without-login persistence deferred to owner).

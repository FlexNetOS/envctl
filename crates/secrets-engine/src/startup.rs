//! Startup-refusal guards for SERVER-MODE Profile B (audit F7/F9; FS-S21/S22/S23/S24).
//!
//! These are the engine-side, **sync, pure** predicates the daemon calls at bring-up BEFORE it
//! begins serving. Each returns `Ok(())` when it can PROVE the configuration is safe, or
//! [`StartupRefusal`] when it cannot — the daemon maps an `Err` to a fail-closed `bail!` (refuse to
//! start). They are pure functions over already-resolved inputs (topology, enrolled keyslots,
//! authorizer config, the gate's resolved state) so they unit-test without a live daemon.
//!
//! Why engine-side: the refusal POLICY (what proves Profile-B safety) is a security invariant that
//! must not diverge between front-ends; the daemon supplies the I/O-resolved inputs and the engine
//! decides. No printing, no I/O here.

use crate::broker::{GateState, PresenceGate};
use crate::keyslot::Factor;
use crate::seam::TrustedTime;
use crate::Topology;

/// Why the daemon must REFUSE to start (fail-closed). Each variant maps to one audited forbidden
/// state; the daemon `bail!`s with the variant's message rather than serving a downgraded config.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StartupRefusal {
    /// FS-S21: `secretd` is configured for VPS (Profile B) but no substitute presence factor (an
    /// operator-authorizer URL) is configured — serving would SILENTLY downgrade to no presence
    /// gating at all. Refuse rather than serve ungated egress.
    #[error(
        "VPS topology requires a configured operator-authorizer URL (substitute presence factor); \
         refusing to start with a silently ungated egress gate (FS-S21)"
    )]
    VpsNoSubstituteFactor,
    /// FS-S22: `secretd` runs on-box (Profile A) and the vault has a USB keyslot enrolled, but USB
    /// possession cannot currently be proven (the gate backs nothing). Refuse unless the operator
    /// explicitly elected passphrase-only operation.
    #[error(
        "on-box topology has a USB keyslot enrolled but USB possession is unproven; pass \
         --allow-passphrase-only to serve passphrase-only, else refusing (FS-S22)"
    )]
    OnBoxUsbKeyslotUnproven,
    /// FS-S23: VPS startup found the presence gate `Unproven` (no valid operator-box token) — a DEK
    /// unwrapped at boot must not authorize egress with no currently-valid presence token. Refuse to
    /// begin serving until the authorizer link delivers a valid token.
    #[error(
        "VPS topology started with no currently-valid presence token (gate is Unproven); refusing \
         to serve boot-unwrapped egress without proven possession (FS-S23)"
    )]
    VpsGateUnprovenAtStartup,
    /// FS-S24: the configuration tries to gate DEK release on a vTPM, whose isolation is
    /// hypervisor-backed (no hardware boundary). vTPM gating is FORBIDDEN; refuse to start.
    #[error(
        "vTPM-gated DEK release is forbidden (no hardware boundary); refusing to start (FS-S24)"
    )]
    VtpmGatingForbidden,
}

/// FS-S21 — when `topology == Vps`, an operator-authorizer URL MUST be configured (the substitute
/// presence factor). On-box topology imposes no such requirement.
pub fn assert_vps_factor_configured(
    topology: Topology,
    operator_authorizer_url: Option<&str>,
) -> Result<(), StartupRefusal> {
    if topology == Topology::Vps
        && operator_authorizer_url
            .map(str::trim)
            .filter(|u| !u.is_empty())
            .is_none()
    {
        return Err(StartupRefusal::VpsNoSubstituteFactor);
    }
    Ok(())
}

/// FS-S22 — on-box (Profile A) with an enrolled USB keyslot but unproven possession refuses, unless
/// the operator passed `--allow-passphrase-only`. A vault with NO USB keyslot is passphrase-only by
/// construction and never trips this. VPS topology is out of scope here (its gate is the authorizer).
pub fn assert_onbox_usb_keyslot_or_override(
    topology: Topology,
    has_enabled_usb_keyslot: bool,
    usb_possession_proven: bool,
    allow_passphrase_only: bool,
) -> Result<(), StartupRefusal> {
    if topology == Topology::OnBox
        && has_enabled_usb_keyslot
        && !usb_possession_proven
        && !allow_passphrase_only
    {
        return Err(StartupRefusal::OnBoxUsbKeyslotUnproven);
    }
    Ok(())
}

/// FS-S23 — on a VPS, the presence gate MUST NOT be `Unproven` at startup (a boot-unwrapped DEK may
/// not serve egress with no valid token). On-box topology is exempt (its gate is the live USB probe,
/// which can legitimately be absent at boot — egress simply denies until the USB is present).
pub fn assert_gate_not_unproven_at_startup(
    topology: Topology,
    gate: &dyn PresenceGate,
) -> Result<(), StartupRefusal> {
    if topology == Topology::Vps && matches!(gate.resolve(), GateState::Unproven) {
        return Err(StartupRefusal::VpsGateUnprovenAtStartup);
    }
    Ok(())
}

/// FS-S24 — vTPM-gated DEK release is forbidden everywhere. The daemon passes whether its config
/// requested vTPM gating (a config-parse-level reject also lives in `secretd::config`, but this is
/// the engine-side invariant so the policy can't diverge).
pub fn assert_no_vtpm_gating(vtpm_gating_requested: bool) -> Result<(), StartupRefusal> {
    if vtpm_gating_requested {
        return Err(StartupRefusal::VtpmGatingForbidden);
    }
    Ok(())
}

/// Convenience: does this keyslot set contain an enabled USB factor? (FS-S22 input.)
#[must_use]
pub fn has_enabled_usb_keyslot(slots: &[crate::keyslot::Keyslot]) -> bool {
    slots.iter().any(|s| s.enabled && s.factor == Factor::Usb)
}

/// OI-SM-3 input helper: trusted time must be available for a VPS to issue/accept presence tokens.
/// (The daemon may surface this as a soft warning at startup; the hard refusal is per-token in
/// [`crate::broker::verify_presence_token`].)
#[must_use]
pub fn trusted_time_available(trusted_time: &dyn TrustedTime) -> bool {
    trusted_time.now_ms().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::{UnprovenGate, VpsPresenceGate};
    use crate::seam::{Clock, SystemClockTrustedTime};

    struct FixedClock(i64);
    impl Clock for FixedClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::from_timestamp_millis(self.0).unwrap()
        }
        fn boottime_ms(&self) -> i64 {
            0
        }
    }

    // ---- FS-S21 -------------------------------------------------------------------------------
    #[test]
    fn vps_without_authorizer_url_refuses() {
        assert_eq!(
            assert_vps_factor_configured(Topology::Vps, None),
            Err(StartupRefusal::VpsNoSubstituteFactor)
        );
        assert_eq!(
            assert_vps_factor_configured(Topology::Vps, Some("   ")),
            Err(StartupRefusal::VpsNoSubstituteFactor),
            "blank URL is no URL"
        );
    }
    #[test]
    fn vps_with_authorizer_url_ok_and_onbox_exempt() {
        assert_eq!(
            assert_vps_factor_configured(Topology::Vps, Some("https://operator.box:9443")),
            Ok(())
        );
        assert_eq!(
            assert_vps_factor_configured(Topology::OnBox, None),
            Ok(()),
            "on-box needs no substitute factor"
        );
    }

    // ---- FS-S22 -------------------------------------------------------------------------------
    #[test]
    fn onbox_usb_keyslot_unproven_refuses_without_override() {
        assert_eq!(
            assert_onbox_usb_keyslot_or_override(Topology::OnBox, true, false, false),
            Err(StartupRefusal::OnBoxUsbKeyslotUnproven)
        );
    }
    #[test]
    fn onbox_usb_keyslot_unproven_allowed_with_override() {
        assert_eq!(
            assert_onbox_usb_keyslot_or_override(Topology::OnBox, true, false, true),
            Ok(()),
            "--allow-passphrase-only lets a passphrase-only operator serve"
        );
    }
    #[test]
    fn onbox_usb_proven_or_no_keyslot_ok() {
        // USB present ⇒ ok even without override.
        assert_eq!(
            assert_onbox_usb_keyslot_or_override(Topology::OnBox, true, true, false),
            Ok(())
        );
        // No USB keyslot at all ⇒ passphrase-only by construction ⇒ ok.
        assert_eq!(
            assert_onbox_usb_keyslot_or_override(Topology::OnBox, false, false, false),
            Ok(())
        );
    }

    // ---- FS-S23 -------------------------------------------------------------------------------
    #[test]
    fn vps_gate_unproven_at_startup_refuses() {
        let gate = UnprovenGate;
        assert_eq!(
            assert_gate_not_unproven_at_startup(Topology::Vps, &gate),
            Err(StartupRefusal::VpsGateUnprovenAtStartup)
        );
    }
    #[test]
    fn vps_gate_present_at_startup_ok() {
        let gate = VpsPresenceGate::new(Box::new(FixedClock(1_000)));
        gate.accept_token(5_000);
        assert_eq!(
            assert_gate_not_unproven_at_startup(Topology::Vps, &gate),
            Ok(())
        );
    }
    #[test]
    fn onbox_gate_unproven_is_exempt() {
        let gate = UnprovenGate;
        assert_eq!(
            assert_gate_not_unproven_at_startup(Topology::OnBox, &gate),
            Ok(()),
            "on-box gate may be legitimately absent at boot"
        );
    }

    // ---- FS-S24 -------------------------------------------------------------------------------
    #[test]
    fn vtpm_gating_refuses() {
        assert_eq!(
            assert_no_vtpm_gating(true),
            Err(StartupRefusal::VtpmGatingForbidden)
        );
        assert_eq!(assert_no_vtpm_gating(false), Ok(()));
    }

    // ---- OI-SM-3 helper -----------------------------------------------------------------------
    #[test]
    fn trusted_time_available_helper() {
        assert!(trusted_time_available(&SystemClockTrustedTime));
    }
}

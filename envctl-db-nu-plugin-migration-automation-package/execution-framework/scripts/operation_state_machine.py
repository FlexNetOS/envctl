from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any


class OperationState(str, Enum):
    PLANNED = "planned"
    RUNNABLE = "runnable"
    APPROVED = "approved"
    RUNNING = "running"
    BLOCKED = "blocked"
    FAILED = "failed"
    COMPLETED = "completed"
    ROLLED_BACK = "rolled_back"


class OperationTransitionError(ValueError):
    pass


RISK_ORDER = {
    "R0": 0,
    "R1": 1,
    "R2": 2,
    "R3": 3,
    "R4": 4,
    "R5": 5,
}

DB_STATUS_BY_STATE = {
    OperationState.PLANNED: "queued",
    OperationState.RUNNABLE: "ready",
    OperationState.APPROVED: "ready",
    OperationState.RUNNING: "running",
    OperationState.BLOCKED: "blocked",
    OperationState.FAILED: "failed",
    OperationState.COMPLETED: "succeeded",
    OperationState.ROLLED_BACK: "cancelled",
}

STATE_BY_DB_STATUS = {
    "queued": OperationState.PLANNED,
    "ready": OperationState.RUNNABLE,
    "awaiting_approval": OperationState.BLOCKED,
    "running": OperationState.RUNNING,
    "succeeded": OperationState.COMPLETED,
    "failed": OperationState.FAILED,
    "blocked": OperationState.BLOCKED,
    "denied": OperationState.BLOCKED,
    "cancelled": OperationState.ROLLED_BACK,
}

TRANSITIONS = {
    OperationState.PLANNED: {
        OperationState.RUNNABLE,
        OperationState.APPROVED,
        OperationState.BLOCKED,
        OperationState.FAILED,
    },
    OperationState.RUNNABLE: {
        OperationState.APPROVED,
        OperationState.RUNNING,
        OperationState.BLOCKED,
        OperationState.FAILED,
    },
    OperationState.APPROVED: {
        OperationState.RUNNING,
        OperationState.BLOCKED,
        OperationState.FAILED,
    },
    OperationState.RUNNING: {
        OperationState.COMPLETED,
        OperationState.BLOCKED,
        OperationState.FAILED,
        OperationState.ROLLED_BACK,
    },
    OperationState.BLOCKED: {
        OperationState.RUNNABLE,
        OperationState.APPROVED,
        OperationState.FAILED,
        OperationState.ROLLED_BACK,
    },
    OperationState.FAILED: {
        OperationState.ROLLED_BACK,
    },
    OperationState.COMPLETED: {
        OperationState.ROLLED_BACK,
    },
    OperationState.ROLLED_BACK: set(),
}

TRIGGER_TARGETS = {
    "mark_runnable": OperationState.RUNNABLE,
    "approve": OperationState.APPROVED,
    "start": OperationState.RUNNING,
    "block": OperationState.BLOCKED,
    "fail": OperationState.FAILED,
    "complete": OperationState.COMPLETED,
    "rollback": OperationState.ROLLED_BACK,
}

TERMINAL_STATES = {
    OperationState.COMPLETED,
    OperationState.ROLLED_BACK,
}


@dataclass(frozen=True)
class OperationRecord:
    operation_id: str
    run_id: str
    operation_type: str
    state: OperationState
    risk: str

    @classmethod
    def planned(
        cls,
        operation_id: str,
        run_id: str,
        operation_type: str,
        risk: str,
    ) -> "OperationRecord":
        validate_risk(risk)
        return cls(
            operation_id=operation_id,
            run_id=run_id,
            operation_type=operation_type,
            state=OperationState.PLANNED,
            risk=risk,
        )


@dataclass(frozen=True)
class TransitionResult:
    operation_id: str
    run_id: str
    operation_type: str
    previous_state: str
    state: str
    trigger: str
    risk: str
    db_status: str
    event_type: str
    terminal: bool
    reason: str | None = None

    def operation_update(self) -> dict[str, Any]:
        update: dict[str, Any] = {"status": self.db_status}
        if self.state == OperationState.RUNNING.value:
            update["started_at_utc"] = "now"
        if self.terminal:
            update["completed_at_utc"] = "now"
        if self.state in {OperationState.BLOCKED.value, OperationState.FAILED.value} and self.reason:
            update["error_json"] = {"reason": self.reason}
        return update

    def event_payload(self) -> dict[str, Any]:
        return {
            "operation_id": self.operation_id,
            "operation_type": self.operation_type,
            "previous_state": self.previous_state,
            "state": self.state,
            "trigger": self.trigger,
            "risk": self.risk,
            "db_status": self.db_status,
            "reason": self.reason,
            "terminal": self.terminal,
        }

    def as_dict(self) -> dict[str, Any]:
        return {
            "operation_id": self.operation_id,
            "run_id": self.run_id,
            "operation_type": self.operation_type,
            "previous_state": self.previous_state,
            "state": self.state,
            "trigger": self.trigger,
            "risk": self.risk,
            "db_status": self.db_status,
            "event_type": self.event_type,
            "terminal": self.terminal,
            "reason": self.reason,
            "operation_update": self.operation_update(),
            "event_payload": self.event_payload(),
        }


def validate_risk(risk: str) -> None:
    if risk not in RISK_ORDER:
        raise OperationTransitionError(f"unknown operation risk: {risk}")


def normalize_state(value: str | OperationState) -> OperationState:
    if isinstance(value, OperationState):
        return value
    normalized = value.replace("-", "_")
    for state in OperationState:
        if normalized == state.value:
            return state
    if value in STATE_BY_DB_STATUS:
        return STATE_BY_DB_STATUS[value]
    raise OperationTransitionError(f"unknown operation state/status: {value}")


def db_status_for_state(state: str | OperationState) -> str:
    return DB_STATUS_BY_STATE[normalize_state(state)]


def approval_required(risk: str) -> bool:
    validate_risk(risk)
    return RISK_ORDER[risk] >= RISK_ORDER["R3"]


def transition(
    record: OperationRecord,
    trigger: str,
    *,
    reason: str | None = None,
) -> tuple[OperationRecord, TransitionResult]:
    if trigger not in TRIGGER_TARGETS:
        raise OperationTransitionError(f"unknown operation trigger: {trigger}")

    target = TRIGGER_TARGETS[trigger]
    if target not in TRANSITIONS[record.state]:
        raise OperationTransitionError(
            f"illegal operation transition: {record.state.value} -> {target.value}"
        )

    if trigger == "start" and approval_required(record.risk) and record.state != OperationState.APPROVED:
        raise OperationTransitionError(
            f"risk {record.risk} operation requires approved state before running"
        )

    next_record = OperationRecord(
        operation_id=record.operation_id,
        run_id=record.run_id,
        operation_type=record.operation_type,
        state=target,
        risk=record.risk,
    )
    result = TransitionResult(
        operation_id=record.operation_id,
        run_id=record.run_id,
        operation_type=record.operation_type,
        previous_state=record.state.value,
        state=target.value,
        trigger=trigger,
        risk=record.risk,
        db_status=DB_STATUS_BY_STATE[target],
        event_type=f"operation_{target.value}",
        terminal=target in TERMINAL_STATES,
        reason=reason,
    )
    return next_record, result


def state_machine_model() -> dict[str, Any]:
    return {
        "canonical_states": [state.value for state in OperationState],
        "terminal_states": [state.value for state in sorted(TERMINAL_STATES, key=lambda item: item.value)],
        "risk_classes": list(RISK_ORDER),
        "approval_required_from_risk": "R3",
        "db_status_by_state": {state.value: status for state, status in DB_STATUS_BY_STATE.items()},
        "state_by_db_status": {status: state.value for status, state in STATE_BY_DB_STATUS.items()},
        "transitions": {
            state.value: sorted(target.value for target in targets)
            for state, targets in TRANSITIONS.items()
        },
        "triggers": {trigger: state.value for trigger, state in TRIGGER_TARGETS.items()},
    }

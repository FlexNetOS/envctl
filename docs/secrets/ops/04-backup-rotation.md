# Backup and rotation

Backups and rotations operate on the canonical Meta payload only. Vault material,
libSQL data, PostgreSQL/RuVector data, ICM SQLite data, and audit state retain their
declared absolute roots beneath `/home/flexnetos/meta/var`.

Quiescence is coordinated through the Yazelix-owned stack and Envctl APIs. Automation
must be added to the Yazelix bootstrap or an owned stack process with explicit
readiness and failure reporting; host timers and login-session state are not runtime
owners. Restore verification must check database integrity, ownership, mode, and the
vault's USB possession requirement before dependents resume.

# SQL feature intent

The SQL files are SQLite-compatible baseline DDL for the migration automation feature. Codex must adapt these to the actual envctl database framework and backend. If envctl uses Postgres, Diesel, SQLx, SeaORM, SurrealDB, sled, or another store, preserve the entities, constraints, indexes, and views in repo-native form.

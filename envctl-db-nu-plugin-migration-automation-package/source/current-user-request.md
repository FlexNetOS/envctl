# Current user request

Create a package that applies the previous migration-artifact response and the `codex-flexnetos-migration-prompt-package` to envctl database + `nu_plugin` automation.

Goal: envctl database should provide built-in migration database tooling controllable by CLI agents. Human users should have live visuals and may choose active involvement or passive supervision. envctl database must be able to reproduce the migration operation anytime, making the migration design faster, more secure, more accurate, and repeatable. The package must be additive to current Codex prompts intended for envctl and nu_plugin GitHub issues.

Required design questions:

1. How should the `codex-flexnetos-migration-prompt-package` be utilized to design database automation feature upgrades?
2. Should Codex implement the package first, build envctl tooling to execute the package first, or do both in parallel?
3. What needs to be added so envctl database can apply the migration to any targets?
4. Find the gaps.
5. Provide a full updated tar.gz package.

# Run and inject UX

`secretctl run` and injection operations talk to the Yazelix-launched `secretd` and
never launch an alternate daemon. The control endpoint, authentication material, and
database locations come from the profile environment front door.

Agents enter through the Yazelix Nushell environment and execute Bash commands
through mandatory RTK interception. JavaScript package execution uses Bun and bunx.
No command may synthesize a home-owned binary directory or a volatile runtime path.

DENIED for dispatch as written.

The packet is not sufficiently bounded to the migration automation package. Although `filesystem_scope` says `package-root`, the `allowed_paths` permit writes across both full repos:

```json
"${ENVCTL_REPO}/**",
"${NU_PLUGIN_REPO}/**"
```

For a high-risk, human-approval-required package automation packet whose goal is to add run/replay templates, that is too broad. Downstream validation is still required in the packet, but broad repo write access means this approval would unlock more mutation surface than the package task needs.

Exact correction instructions:

1. Restrict `allowed_paths` to the package-owned framework/template/proof surfaces only, for example:
   ```json
   [
     "execution-framework/**",
     "execution-templates/**",
     "proof_records/**",
     "state/**",
     "logs/**"
   ]
   ```

2. Replace vague `target_files`:
   ```json
   ["run/replay templates"]
   ```
   with concrete package-relative paths, such as:
   ```json
   [
     "execution-templates/**",
     "execution-framework/**"
   ]
   ```

3. Keep repo source trees read-only inputs only. If envctl or nu_plugin files are needed for template generation, list them under `input_files`, not `allowed_paths`.

4. Preserve the existing downstream gate language: proof file required, task-specific tests required, and no claim that CI or downstream validation has completed.

5. Recompute and provide the new packet sha256 after editing. The reviewed packet sha256 was `d7882d5d6d7f92c705fde6641213be48a2f1f05547a85b75a50822d8fee1d36d`.

I also could not verify the packet on disk in this read-only sandbox because local shell reads returned nonzero without output, so this decision is based on the packet JSON supplied in the request.
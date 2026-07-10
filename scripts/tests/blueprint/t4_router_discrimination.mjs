// canonical: scripts/tests/blueprint/t4_router_discrimination.mjs
// T4 — bun golden-fixture test for model-tier routing discrimination.
//   For every fixture case it invokes the router and asserts the emitted
//   `modelTier` equals the expected tier on ALL stability runs. A tier that is
//   ABSENT from the router output counts as FAIL.
//
//   RED modes (both captured):
//     * default ROUTER_DIR (/home/flexnetos/lifeos/src/meta-ruvector): the plain
//       agent router emits no `modelTier` at all -> every case FAIL (ABSENT).
//     * ROUTER_DIR=/home/flexnetos/lifeos/src/meta-ruvector-router-wt: the RuvLTRA
//       tier layer emits a CONSTANT tier regardless of prompt complexity, so it
//       cannot discriminate -> the opus-expected cases FAIL.
//   GREEN (after R4 calibrates the classifier): all 10 cases discriminate.
//
// Invocation contract (per plan): ROUTER_DIR env (default meta-ruvector);
//   bash -lc 'cd $ROUTER_DIR && bun .claude/helpers/router.js "<prompt>"';
//   parse the emitted JSON, read `field` (default "modelTier").
//
// Env overrides: ROUTER_DIR, T4_FIXTURE (fixture path), T4_LIMIT (>0 caps the
//   number of cases — used only to keep RED captures fast; the committed default
//   runs the FULL fixture).
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROUTER_DIR = process.env.ROUTER_DIR || '/home/flexnetos/lifeos/src/meta-ruvector';
const FIXTURE = process.env.T4_FIXTURE || join(HERE, 'fixtures', 'router_prompts.json');
const LIMIT = process.env.T4_LIMIT ? parseInt(process.env.T4_LIMIT, 10) : 0;

const spec = JSON.parse(readFileSync(FIXTURE, 'utf8'));
const field = spec.field || 'modelTier';
const runs = spec.stability_runs || 3;
let cases = spec.cases || [];
if (LIMIT > 0) cases = cases.slice(0, LIMIT);

// POSIX single-quote so the prompt cannot be re-interpreted by the shell.
const shq = (s) => `'${String(s).replace(/'/g, `'\\''`)}'`;

function routerTier(prompt) {
  const cmd = `cd ${shq(ROUTER_DIR)} && bun .claude/helpers/router.js ${shq(prompt)}`;
  let out;
  try {
    out = execFileSync('bash', ['-lc', cmd], { encoding: 'utf8', timeout: 20000 });
  } catch (e) {
    return { err: String(e.stderr || e.message || 'spawn failed').slice(0, 160) };
  }
  const s = out.indexOf('{');
  const t = out.lastIndexOf('}');
  if (s < 0 || t <= s) return { err: 'no JSON object in router output' };
  let parsed;
  try {
    parsed = JSON.parse(out.slice(s, t + 1));
  } catch (e) {
    return { err: `JSON parse: ${e.message}` };
  }
  return { tier: parsed[field] };
}

console.log(`T4 router discrimination — ROUTER_DIR=${ROUTER_DIR} field=${field} stability_runs=${runs} cases=${cases.length}`);
let failures = 0;
for (const c of cases) {
  const observed = [];
  let caseFail = false;
  for (let i = 0; i < runs; i += 1) {
    const r = routerTier(c.prompt);
    if (r.err) {
      observed.push(`ERR(${r.err})`);
      caseFail = true;
    } else if (r.tier === undefined || r.tier === null) {
      observed.push('ABSENT');
      caseFail = true;
    } else {
      observed.push(r.tier);
      if (r.tier !== c.expect_tier) caseFail = true;
    }
  }
  const label = `[expect ${c.expect_tier}] "${c.prompt.slice(0, 64)}"`;
  if (caseFail) {
    failures += 1;
    console.log(`FAIL ${label} -> ${JSON.stringify(observed)}`);
  } else {
    console.log(`PASS ${label} -> ${JSON.stringify(observed)}`);
  }
}

if (failures === 0) {
  console.log(`T4 GREEN: ${cases.length}/${cases.length} cases discriminated across ${runs} stability runs`);
  process.exit(0);
} else {
  console.log(`T4 RED: ${failures}/${cases.length} cases failed`);
  process.exit(1);
}

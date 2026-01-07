# job.vit — Muffin (Vitte) — MAX
#
# Job model + scheduler planning primitives for Muffin/Steel.
# -----------------------------------------------------------------------------
# Concepts:
# - A Job is an executable unit derived from a Step (already resolved + expanded).
# - A Job has:
#   - stable id (hash) for cache keys
#   - inputs/outputs (files/dirs)
#   - command argv + cwd + env
#   - dependency edges to other jobs (DAG)
#   - cache policy (read/write/disabled)
#   - execution policy (local/remote, timeout, retries)
#
# This module DOES NOT execute processes; it only models:
# - job graph construction helpers
# - topological scheduling order
# - "ready queue" selection policy (deterministic)
# - incremental cache checks (interface-driven)
#
# Pipeline:
#   resolve -> implicit -> expand -> hash -> plan -> jobs -> exec
#
# Blocks: .end only
# -----------------------------------------------------------------------------

mod muffin/job

use std/string
use std/result
use muffin/config
use muffin/hash
use muffin/interface
use muffin/directory
use muffin/debug
use muffin/externs

export all

# -----------------------------------------------------------------------------
# Errors / Diagnostics
# -----------------------------------------------------------------------------

enum JobErrKind
  Invalid
  Cycle
  UnknownDep
  Hash
  Cache
.end

struct JobError
  kind: JobErrKind
  message: str
  name: str
.end

type JobRes[T] = result::Result[T, JobError]

fn job_err(kind: JobErrKind, msg: str, name: str) -> JobError
  ret JobError(kind: kind, message: msg, name: name)
.end

# -----------------------------------------------------------------------------
# Policies
# -----------------------------------------------------------------------------

enum CacheMode
  Disabled
  ReadOnly
  WriteOnly
  ReadWrite
.end

enum ExecMode
  Local
  Remote
  Auto
.end

struct RetryPolicy
  retries: i32
  backoff_ms: i64
.end

fn retry_default() -> RetryPolicy
  ret RetryPolicy(retries: 0, backoff_ms: 0)
.end

struct JobPolicy
  cache: CacheMode
  exec: ExecMode
  timeout_ms: i64
  retry: RetryPolicy
  allow_parallel: bool
.end

fn policy_default() -> JobPolicy
  ret JobPolicy(
    cache: CacheMode::ReadWrite,
    exec: ExecMode::Local,
    timeout_ms: 0,
    retry: retry_default(),
    allow_parallel: true
  )
.end

# -----------------------------------------------------------------------------
# Job types
# -----------------------------------------------------------------------------

enum JobState
  Pending
  Ready
  Running
  Done
  Failed
  Skipped
.end

struct JobIo
  inputs: list[str]
  outputs: list[str]
.end

struct JobCmd
  argv: list[str]
  cwd: str
  env: map[str, str]
.end

struct Job
  id: str                # stable digest "sha256:..."
  name: str              # target.step
  target: str
  step: str

  policy: JobPolicy

  io: JobIo
  cmd: JobCmd

  deps: list[str]        # job ids
  users: list[str]       # reverse edges (job ids)

  state: JobState

  # runtime fields (filled by executor)
  exit_code: i32
  duration_ms: i64
.end

fn job_new(id: str, name: str, target: str, step: str, pol: JobPolicy, io: JobIo, cmd: JobCmd) -> Job
  ret Job(
    id: id,
    name: name,
    target: target,
    step: step,
    policy: pol,
    io: io,
    cmd: cmd,
    deps: [],
    users: [],
    state: JobState::Pending,
    exit_code: 0,
    duration_ms: 0
  )
.end

# -----------------------------------------------------------------------------
# Graph container
# -----------------------------------------------------------------------------

struct JobGraph
  jobs: list[Job]
  # index: id -> position
  idx: map[str, i32]
.end

fn graph_new() -> JobGraph
  ret JobGraph(jobs: [], idx: map_new_i32())
.end

fn graph_has(g: JobGraph, id: str) -> bool
  ret map_has_i32(g.idx, id)
.end

fn graph_get_index(g: JobGraph, id: str) -> i32
  if !map_has_i32(g.idx, id) ret -1 .end
  ret map_get_i32(g.idx, id)
.end

fn graph_get(g: JobGraph, id: str) -> Job
  let i: i32 = graph_get_index(g, id)
  ret g.jobs[i]
.end

fn graph_put(mut g: JobGraph, j: Job) -> JobGraph
  if map_has_i32(g.idx, j.id)
    # overwrite not allowed (stability)
    return g
  .end
  g.idx = map_put_i32(g.idx, j.id, len(g.jobs))
  g.jobs = g.jobs + [j]
  return g
.end

fn graph_add_edge(mut g: JobGraph, from_id: str, to_id: str) -> JobRes[JobGraph]
  # from -> to means: to depends on from
  let fi: i32 = graph_get_index(g, from_id)
  let ti: i32 = graph_get_index(g, to_id)
  if fi < 0 ret result::Err(job_err(JobErrKind::UnknownDep, "missing job: " + from_id, to_id)) .end
  if ti < 0 ret result::Err(job_err(JobErrKind::UnknownDep, "missing job: " + to_id, from_id)) .end

  let mut a: Job = g.jobs[fi]
  let mut b: Job = g.jobs[ti]

  if !contains_str(b.deps, from_id)
    b.deps = b.deps + [from_id]
  .end
  if !contains_str(a.users, to_id)
    a.users = a.users + [to_id]
  .end

  g.jobs[fi] = a
  g.jobs[ti] = b
  ret result::Ok(g)
.end

# -----------------------------------------------------------------------------
# Build jobs from resolved config
# -----------------------------------------------------------------------------
# Assumptions:
# - implicit + expand already applied
# - tools resolved, argv stable
# - inputs/outputs known
# - hashing module available

struct BuildJobsOptions
  policy: JobPolicy
  include_targets: list[str]    # if empty => selection target only
  max_jobs: i32
.end

fn build_jobs_options_default() -> BuildJobsOptions
  ret BuildJobsOptions(policy: policy_default(), include_targets: [], max_jobs: 100000)
.end

fn jobs_from_resolved(rt: Runtime, r: Resolved) -> JobRes[JobGraph]
  let opt: BuildJobsOptions = build_jobs_options_default()
  ret jobs_from_resolved_with(rt, r, opt)
.end

fn jobs_from_resolved_with(rt: Runtime, r: Resolved, opt: BuildJobsOptions) -> JobRes[JobGraph]
  let algo: hash::HashAlgo = hash::default_algo()
  let mut g: JobGraph = graph_new()

  # target selection list
  let targets: list[Target] = select_targets(r, opt.include_targets)

  # 1) create jobs
  let mut created: i32 = 0
  let mut ti: i32 = 0
  while ti < len(targets)
    let t: Target = targets[ti]
    ti = ti + 1

    let mut si: i32 = 0
    while si < len(t.steps)
      if created >= opt.max_jobs
        ret result::Err(job_err(JobErrKind::Invalid, "max_jobs reached", "jobs_from_resolved"))
      .end

      let s: Step = t.steps[si]
      let jname: str = t.name + "." + s.name

      # compute job key
      let env2: map[str, str] = s.env
      let rr: hash::HashRes[hash::Digest] =
        hash::hash_step_key(algo, s.tool, s.argv, s.cwd, env2, s.inputs, s.outputs)

      if result::is_err(rr)
        ret result::Err(job_err(JobErrKind::Hash, "hash_step_key failed", jname))
      .end

      let id: str = hash::digest_str(result::unwrap(rr))

      let io: JobIo = JobIo(inputs: s.inputs, outputs: s.outputs)
      let cmd: JobCmd = JobCmd(argv: s.argv, cwd: s.cwd, env: env2)

      let j: Job = job_new(id, jname, t.name, s.name, opt.policy, io, cmd)
      g = graph_put(g, j)
      created = created + 1

      si = si + 1
    .end
  .end

  # 2) add edges:
  # - sequential edges within a target (step[i] -> step[i+1])
  # - target deps: all jobs in dep target -> first job in target (or all jobs)
  let rr2: JobRes[JobGraph] = wire_edges(r, targets, g)
  if result::is_err(rr2) ret rr2 .end
  g = result::unwrap(rr2)

  # 3) validate acyclic
  let rr3: JobRes[list[str]] = topo_order(g)
  if result::is_err(rr3) ret rr3 as JobGraph .end

  # 4) mark ready jobs (deps empty)
  g = mark_initial_ready(g)

  ret result::Ok(g)
.end

fn select_targets(r: Resolved, include: list[str]) -> list[Target]
  if len(include) == 0
    let idx: i32 = find_target(r.targets, r.selection.target)
    if idx < 0
      return []
    .end
    return [r.targets[idx]]
  .end

  let mut out: list[Target] = []
  let mut i: i32 = 0
  while i < len(include)
    let n: str = include[i]
    let j: i32 = find_target(r.targets, n)
    if j >= 0 out = out + [r.targets[j]] .end
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Edge wiring
# -----------------------------------------------------------------------------

fn wire_edges(r: Resolved, targets: list[Target], g0: JobGraph) -> JobRes[JobGraph]
  let mut g: JobGraph = g0

  # Step sequencing within a target:
  let mut ti: i32 = 0
  while ti < len(targets)
    let t: Target = targets[ti]
    ti = ti + 1

    # map step index -> job id
    let ids: list[str] = job_ids_for_target(g, t.name)

    let mut i: i32 = 0
    while i + 1 < len(ids)
      let a: str = ids[i]
      let b: str = ids[i + 1]
      let rr: JobRes[JobGraph] = graph_add_edge(g, a, b)
      if result::is_err(rr) ret rr .end
      g = result::unwrap(rr)
      i = i + 1
    .end

    # target deps -> this target's first step
    if len(ids) > 0
      let first: str = ids[0]
      let mut d: i32 = 0
      while d < len(t.deps)
        let dep_name: str = t.deps[d]
        d = d + 1

        let dep_ids: list[str] = job_ids_for_target(g, dep_name)
        if len(dep_ids) == 0
          # dep target with no steps => ignore (phony)
          continue
        .end

        # all jobs in dep must complete before first job here
        let mut k: i32 = 0
        while k < len(dep_ids)
          let rr2: JobRes[JobGraph] = graph_add_edge(g, dep_ids[k], first)
          if result::is_err(rr2) ret rr2 .end
          g = result::unwrap(rr2)
          k = k + 1
        .end
      .end
    .end
  .end

  ret result::Ok(g)
.end

fn job_ids_for_target(g: JobGraph, target: str) -> list[str]
  # stable order: by job.name
  let mut tmp: list[str] = []
  let mut i: i32 = 0
  while i < len(g.jobs)
    let j: Job = g.jobs[i]
    if j.target == target
      tmp = tmp + [j.id]
    .end
    i = i + 1
  .end
  # sort by job.name (need lookup)
  tmp = sort_ids_by_name(g, tmp)
  ret tmp
.end

fn sort_ids_by_name(g: JobGraph, ids: list[str]) -> list[str]
  let mut a: list[str] = ids
  let mut i: i32 = 0
  while i < len(a)
    let mut j: i32 = i + 1
    while j < len(a)
      let ni: str = g.jobs[graph_get_index(g, a[i])].name
      let nj: str = g.jobs[graph_get_index(g, a[j])].name
      if nj < ni
        let t: str = a[i]
        a[i] = a[j]
        a[j] = t
      .end
      j = j + 1
    .end
    i = i + 1
  .end
  ret a
.end

# -----------------------------------------------------------------------------
# Topological order / cycle detection (Kahn)
# -----------------------------------------------------------------------------

fn topo_order(g: JobGraph) -> JobRes[list[str]]
  # indegree map
  let mut indeg: map[str, i32] = map_new_i32()
  let mut i: i32 = 0
  while i < len(g.jobs)
    let id: str = g.jobs[i].id
    indeg = map_put_i32(indeg, id, 0)
    i = i + 1
  .end

  i = 0
  while i < len(g.jobs)
    let j: Job = g.jobs[i]
    let mut k: i32 = 0
    while k < len(j.deps)
      let dep: str = j.deps[k]
      if !map_has_i32(indeg, dep)
        ret result::Err(job_err(JobErrKind::UnknownDep, "unknown dep id: " + dep, j.name))
      .end
      indeg = map_put_i32(indeg, j.id, map_get_i32(indeg, j.id) + 1)
      k = k + 1
    .end
    i = i + 1
  .end

  # queue of zero indegree, deterministic by name
  let mut q: list[str] = []
  i = 0
  while i < len(g.jobs)
    let id: str = g.jobs[i].id
    if map_get_i32(indeg, id) == 0 q = q + [id] .end
    i = i + 1
  .end
  q = sort_ids_by_name(g, q)

  let mut out: list[str] = []
  while len(q) > 0
    let id: str = q[0]
    q = slice_str(q, 1, len(q))
    out = out + [id]

    let j: Job = g.jobs[graph_get_index(g, id)]
    let mut u: i32 = 0
    while u < len(j.users)
      let v: str = j.users[u]
      indeg = map_put_i32(indeg, v, map_get_i32(indeg, v) - 1)
      if map_get_i32(indeg, v) == 0
        q = q + [v]
        q = sort_ids_by_name(g, q)  # deterministic
      .end
      u = u + 1
    .end
  .end

  if len(out) != len(g.jobs)
    ret result::Err(job_err(JobErrKind::Cycle, "cycle detected in job graph", "topo_order"))
  .end

  ret result::Ok(out)
.end

# -----------------------------------------------------------------------------
# Ready marking + scheduling helpers
# -----------------------------------------------------------------------------

fn mark_initial_ready(mut g: JobGraph) -> JobGraph
  let mut i: i32 = 0
  while i < len(g.jobs)
    let mut j: Job = g.jobs[i]
    if len(j.deps) == 0
      j.state = JobState::Ready
    else
      j.state = JobState::Pending
    .end
    g.jobs[i] = j
    i = i + 1
  .end
  ret g
.end

fn ready_queue(g: JobGraph) -> list[str]
  let mut ids: list[str] = []
  let mut i: i32 = 0
  while i < len(g.jobs)
    let j: Job = g.jobs[i]
    if j.state == JobState::Ready
      ids = ids + [j.id]
    .end
    i = i + 1
  .end
  ids = sort_ids_by_name(g, ids)
  ret ids
.end

fn mark_done(mut g: JobGraph, id: str, exit_code: i32, dur_ms: i64) -> JobGraph
  let idx: i32 = graph_get_index(g, id)
  if idx < 0 return g .end

  let mut j: Job = g.jobs[idx]
  j.state = (exit_code == 0) ? JobState::Done : JobState::Failed
  j.exit_code = exit_code
  j.duration_ms = dur_ms
  g.jobs[idx] = j

  # update users indeps: if all deps done => ready
  let mut u: i32 = 0
  while u < len(j.users)
    let uid: str = j.users[u]
    let ui: i32 = graph_get_index(g, uid)
    if ui >= 0
      let mut uj: Job = g.jobs[ui]
      if uj.state == JobState::Pending
        if deps_all_done(g, uj)
          uj.state = JobState::Ready
          g.jobs[ui] = uj
        .end
      .end
    .end
    u = u + 1
  .end

  ret g
.end

fn deps_all_done(g: JobGraph, j: Job) -> bool
  let mut i: i32 = 0
  while i < len(j.deps)
    let di: i32 = graph_get_index(g, j.deps[i])
    if di < 0 return false .end
    let st: JobState = g.jobs[di].state
    if st != JobState::Done && st != JobState::Skipped
      return false
    .end
    i = i + 1
  .end
  ret true
.end

# -----------------------------------------------------------------------------
# Cache helpers (keying by Job.id)
# -----------------------------------------------------------------------------

fn cache_key_for_job(j: Job) -> str
  ret "job:" + j.id
.end

fn cache_try_hit(rt: Runtime, j: Job) -> JobRes[bool]
  if j.policy.cache == CacheMode::Disabled || j.policy.cache == CacheMode::WriteOnly
    ret result::Ok(false)
  .end
  let key: str = cache_key_for_job(j)
  let rr: interface::CacheRes[bool] = interface::cache_has(rt.cache, key)
  if result::is_err(rr)
    ret result::Err(job_err(JobErrKind::Cache, "cache has failed", j.name))
  .end
  ret result::Ok(result::unwrap(rr))
.end

# -----------------------------------------------------------------------------
# Utils
# -----------------------------------------------------------------------------

fn contains_str(xs: list[str], x: str) -> bool
  let mut i: i32 = 0
  while i < len(xs)
    if xs[i] == x ret true .end
    i = i + 1
  .end
  ret false
.end

fn slice_str(xs: list[str], a: i32, b: i32) -> list[str]
  let mut out: list[str] = []
  let mut i: i32 = a
  while i < b
    out = out + [xs[i]]
    i = i + 1
  .end
  ret out
.end

# -----------------------------------------------------------------------------
# Externs
# -----------------------------------------------------------------------------

extern fn map_new_i32() -> map[str, i32]
extern fn map_put_i32(m: map[str, i32], k: str, v: i32) -> map[str, i32]
extern fn map_get_i32(m: map[str, i32], k: str) -> i32
extern fn map_has_i32(m: map[str, i32], k: str) -> bool

extern fn len[T](xs: list[T]) -> i32

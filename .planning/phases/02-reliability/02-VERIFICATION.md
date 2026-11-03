---
phase: 02-reliability
verified: 2026-11-03T15:28:15Z
status: human_needed
score: 24/24 merged observable truths verified
head: 8d544d7e86fe3eedac5c490e5f08688c5905b746
covered_files: [".github/workflows/ci.yml",".planning/phases/02-reliability/02-01-PLAN.md",".planning/phases/02-reliability/02-01-SUMMARY.md",".planning/phases/02-reliability/02-02-PLAN.md",".planning/phases/02-reliability/02-02-SUMMARY.md",".planning/phases/02-reliability/02-03-PLAN.md",".planning/phases/02-reliability/02-03-SUMMARY.md",".planning/phases/02-reliability/02-04-PLAN.md",".planning/phases/02-reliability/02-04-SUMMARY.md",".planning/phases/02-reliability/02-05-PLAN.md",".planning/phases/02-reliability/02-05-SUMMARY.md",".planning/phases/02-reliability/02-06-PLAN.md",".planning/phases/02-reliability/02-06-SUMMARY.md",".planning/phases/02-reliability/02-07-PLAN.md",".planning/phases/02-reliability/02-07-SUMMARY.md",".planning/phases/02-reliability/02-08-PLAN.md",".planning/phases/02-reliability/02-08-SUMMARY.md",".planning/phases/02-reliability/02-09-PLAN.md",".planning/phases/02-reliability/02-09-SUMMARY.md",".planning/phases/02-reliability/02-10-PLAN.md",".planning/phases/02-reliability/02-10-SUMMARY.md",".planning/phases/02-reliability/02-CONTEXT.md",".planning/phases/02-reliability/02-REVIEW.md","AGENTS.md","Cargo.lock","Cargo.toml","SPEC.md","hk.pkl","mise.toml","src/agent.rs","src/agent/request.rs","src/agent/stop.rs","src/agent/stop/tests.rs","src/app.rs","src/cancel.rs","src/cancel/tests.rs","src/cli.rs","src/config.rs","src/config/error.rs","src/config/error/key.rs","src/config/raw.rs","src/config/tests.rs","src/config/tests/errors.rs","src/config/tests/files.rs","src/context/tests/previous_shapes.rs","src/crash.rs","src/debug.rs","src/error.rs","src/lock.rs","src/lock/role.rs","src/lock/tests.rs","src/main.rs","src/normalize.rs","src/recovery.rs","src/recovery/cleanup.rs","src/recovery/rules.rs","src/recovery/rules/failure.rs","src/recovery/rules/tests.rs","src/recovery/rules/tests/table.rs","src/recovery/scan.rs","src/recovery/tests.rs","src/recovery/tests/budget.rs","src/recovery/tests/causes.rs","src/recovery/tests/fixture.rs","src/recovery/tests/orphan.rs","src/render.rs","src/render/tests.rs","src/render/tests/fixture.rs","src/render/tests/freshness.rs","src/render/tests/retry.rs","src/review.rs","src/review/acquire.rs","src/review/print.rs","src/review/signals.rs","src/review/signals/tests.rs","src/review/wait.rs","src/review/wait/tests.rs","src/runner.rs","src/runner/agent_run.rs","src/runner/agent_run/deadline.rs","src/runner/agent_run/fakes.rs","src/runner/agent_run/fakes/git.rs","src/runner/agent_run/tests.rs","src/runner/agent_run/tests/completion.rs","src/runner/agent_run/tests/edges.rs","src/runner/agent_run/tests/fixture.rs","src/runner/agent_run/tests/persistence.rs","src/runner/agent_run/tests/persistence_run.rs","src/runner/agent_run/timing_tests.rs","src/runner/cancelled.rs","src/runner/outcome.rs","src/runner/outcome/failed.rs","src/runner/outcome/succeeded.rs","src/runner/outcome/tests.rs","src/runner/outcome/tests/fixture.rs","src/runner/outcome/tests/preservation.rs","src/runner/outcome/tests/retry.rs","src/runner/outcome/tests/terminal.rs","src/runner/process.rs","src/runner/resolve.rs","src/runner/resolve/tests.rs","src/runner/resolve/tests/gh.rs","src/runner/resolve/tests/git.rs","src/runner/resolve/tests/reuse.rs","src/runner/resolve/tests/reuse/seed.rs","src/runner/result.rs","src/runner/result/stored.rs","src/runner/run.rs","src/runner/steps.rs","src/runner/tests.rs","src/runner/tests/before.rs","src/status.rs","src/status/budget.rs","src/status/freshness.rs","src/status/tests.rs","src/status/tests/budget.rs","src/status/tests/fixture.rs","src/status/tests/stored.rs","src/status/when.rs","src/store.rs","src/store/finish.rs","src/store/jobs.rs","src/store/jobs/attach.rs","src/store/jobs/attach/active.rs","src/store/jobs/due.rs","src/store/jobs/tests.rs","src/store/jobs/tests/fixture.rs","src/store/migrate.rs","src/store/migrate/backup.rs","src/store/migrate/tests.rs","src/store/migrate/tests/backups.rs","src/store/migrate/tests/concurrency.rs","src/store/migrate/tests/helpers.rs","src/store/migrate/tests/rollback.rs","src/store/migrate_v2.sql","src/store/readonly.rs","src/store/records.rs","src/store/recovery.rs","src/store/recovery/resolve.rs","src/store/result.rs","src/store/result/read.rs","src/store/retry.rs","src/store/retry/queue.rs","src/store/retry/revision.rs","src/store/retry/tests.rs","src/store/runs.rs","src/store/schema.sql","src/store/status.rs","src/store/status/latest.rs","src/store/targets.rs","src/tests/isolation/auth.rs","src/tests/live/agent.rs","src/tests/live/prs.rs","src/tests/live/support.rs","src/watch.rs","src/watch/events.rs","src/watch/events/date.rs","src/watch/events/outcomes.rs","src/watch/events/tests.rs","src/watch/run.rs","src/watch/wait.rs","tests/attach.rs","tests/attach/finished.rs","tests/attach/forced.rs","tests/attach/helpers.rs","tests/concurrency/helpers.rs","tests/config.rs","tests/context.rs","tests/crash.rs","tests/crash/budget.rs","tests/crash/helpers.rs","tests/crash/manual_stop.rs","tests/crash/matrix.rs","tests/crash/matrix/assertions.rs","tests/crash/process.rs","tests/crash/restart.rs","tests/crash/stop_stored.rs","tests/dedup.rs","tests/migration.rs","tests/migration/fixture.rs","tests/migration/fixture.sql","tests/rerun.rs","tests/rerun/cancel.rs","tests/rerun/delayed.rs","tests/review_output.rs","tests/review_output/previous.rs","tests/signals.rs","tests/signals/helpers.rs","tests/signals/process.rs","tests/stale_review.rs","tests/stale_review/edges.rs","tests/status.rs","tests/status/helpers.rs","tests/status_freshness.rs","tests/status_freshness/failure.rs","tests/support/fake_claude.rs","tests/support/fake_claude.sh","tests/support/fake_gh.rs","tests/watch.rs","tests/watch/attach.rs","tests/watch/debug.rs","tests/watch/helpers.rs","tests/watch/persistence.rs","tests/watch/shutdown.rs","tests/watch/wait.rs"]
covered_digest: "v4:sha256:6b8acfa069d63a79d98ae2bdd92207bc2d3da1e1fa1b0a9ff6f598e2ad739a62"
behavior_unverified: 0
overrides_applied: 0
implementation_gaps: 0
roadmap_truths: 11
plan_truths: 76
requirements_covered: 13
artifacts_verified: 34
key_links_verified: 23
prohibitions_test_verified: 13
prohibitions_judgment_pending: 2
phase_gate: blocked
external_gate: Anthropic account verification HTTP 403
decision_coverage:
  honored: 16
  total: 16
  not_honored: []
human_verification:
  - id: UAT-01
    source: "WINDOWS-1"
    test: "На release binary выполнить review реального fixture PR и нажать Ctrl-C во время агента; проверить status и процессы группы."
    expected: "Exit 1, сообщение Run cancelled; recorded as interrupted, failed (interrupted: user_cancel), отсутствие агента и rg этой группы."
    why_human: "Реальный Claude и терминальные сигналы; автоматический двойник использует fake CLI."
  - id: UAT-02
    source: "WINDOWS-2"
    test: "Продвинуть только base ветку fixture PR и выполнить status."
    expected: "Черновик остаётся current при неизменном PR head."
    why_human: "Ручной сценарий REL-03 с GitHub ещё не выполнен."
  - id: UAT-03
    source: "WINDOWS-3"
    test: "Дважды выполнить review одного head, затем --force; параллельно запустить два review второго PR."
    expected: "Повтор явно переиспользует run; --force создаёт новый; одновременные обычные запросы получают один run."
    why_human: "Нужна проверка вывода и пользовательского потока release CLI."
  - id: UAT-04
    source: "WINDOWS-4"
    test: "Во время реального review отправить commit в PR, затем прочитать результат и status."
    expected: "Exit 0; первая строка review и status явно говорят stale."
    why_human: "Ручной push-during-run с реальным GitHub и агентом ещё не выполнен."
  - id: UAT-05
    source: "WINDOWS-5"
    test: "На release binary проверить run_timeout_minutes = 1 с review длительнее минуты, затем вернуть настройку."
    expected: "Failed (timeout), exit 1, очищенный worktree и отсутствие процессов агента этой группы."
    why_human: "Точный deadline доказан paused-time тестами, реальный Claude timeout пока не наблюдался."
  - id: UAT-06
    source: "WINDOWS-7"
    test: "Поставить automatic job через debug enqueue, запустить watch, отправить SIGTERM во время агента и перезапустить watch."
    expected: "Shutdown записан, worktree удалён, job requeued без расхода бюджета и затем выполнен."
    why_human: "Release/live сценарий REL-08 остаётся открытым."
  - id: UAT-07
    source: "WINDOWS-8"
    test: "Без watch поставить manual re-run URL_A и automatic URL_B; выполнить review URL_A."
    expected: "Review присоединяется к A и исполняет только A; B остаётся queued без run."
    why_human: "Ручной сценарий REL-10 на release binary ещё не выполнен."
  - id: UAT-08
    source: "WINDOWS-9"
    test: "На release binary трижды убить watch через SIGKILL во время одного automatic review, перезапуская его между попытками."
    expected: "Status показывает retry 1 of 2, retry 2 of 2, затем no retries left; четвёртой попытки нет."
    why_human: "Процессная crash-матрица использует fake gh/claude; live проверка D-16 открыта."
  - id: UAT-09
    source: "WINDOWS-10"
    test: "На release binary убить review во время подготовки и во время агента; после каждого сбоя повторить review."
    expected: "Повтор успешен, существовавший run помечен interrupted: crash, runs directory пуст и git worktree metadata содержит только bare clone."
    why_human: "План D-13 отдельно требует live SIGKILL поверх feature-build матрицы."
  - id: UAT-10
    source: "02-01 Task 3 human-check"
    test: "После отдельной копии реальной Phase 1 базы выполнить release status, review fixture PR и status; сравнить прежние данные и v1-backup."
    expected: "Прежние target/job/run/draft/finding значения сохранены, backup валиден; outdated-schema hint до первого upgrade, новые данные доступны после."
    why_human: "Проверка реальной владельческой базы не выполнена и отсутствовала в WINDOWS; одобрение additive-backup не заменяет этот hand check."
  - id: LIVE-01
    source: "WINDOWS-11"
    test: "После восстановления доступа к Anthropic по отдельному указанию владельца повторить isolation_flags_keep_authentication."
    expected: "Настоящий Claude работает с production isolation flags, без HTTP 403."
    why_human: "Сейчас внешний account-verification барьер. Владелец выбрал сохранить live gate заблокированным; это не waiver и не разрешение на автоматический повтор."
  - id: LIVE-02
    source: "WINDOWS-12"
    test: "После восстановления account access выполнить четыре оставшихся model-backed случая через mise run test:e2e."
    expected: "Max-turns даёт failed, clean PR — no_findings, bug и fork PR — ожидаемую находку calc.py:7."
    why_human: "Четыре проверки не запускались после fail-fast на auth; локальные fakes не доказывают модельный результат."
  - id: JUDGMENT-01
    source: "02-03 prohibition"
    test: "При итоговом UAT подтвердить проверенную границу сигналов: только group текущего локально порождённого agent child, никогда PID/PGID из SQLite."
    expected: "Сохранённые идентификаторы служат диагностикой; recovery не сигналит их."
    why_human: "План помечает запрет verification: judgment. Исходники соответствуют, но LLM-review не является явным human resolution."
  - id: JUDGMENT-02
    source: "02-09 prohibition"
    test: "При итоговом UAT подтвердить границу watch этой фазы: потребление существующих review jobs без собственного polling/producer."
    expected: "Watch не создаёт jobs и не опрашивает GitHub вне исполняемого review."
    why_human: "План помечает запрет verification: judgment. Проверка кода подтверждает границу; явное human resolution ещё не записано."
---

# Phase 2: Reliability — отчёт верификации

**Цель:** review можно прервать, повторить или застать новым push без дублирующих запусков, потерянной причины остановки, осиротевших worktree и неограниченных повторов; черновик сообщает соответствие PR head.

**Статус: human_needed.** Дефектов реализации, препятствующих цели Phase 2, не обнаружено. Это **не** завершённая фаза: обязательный live-gate заблокирован Anthropic HTTP 403, десять release/live hand checks не выполнены, два judgment-tier запрета ожидают подтверждения в итоговом UAT. Решение владельца «Сохранить live gate как заблокированный» сохранено без waiver. Новых Claude API вызовов не было.

Проверка первичная, на HEAD `8d544d7`. Прочитаны SPEC, все десять PLAN/SUMMARY, CONTEXT, REQUIREMENTS, предшествующий 01-VERIFICATION, WINDOWS и 02-REVIEW. SPEC имеет приоритет над планами. Проверены текущие реализации, места вызова и assertions; утверждения SUMMARY сами по себе не считались доказательством.

## Происхождение доказательств

- **V — непосредственно эта проверка:** чтение исходников и assertions; CLI-проверки artifacts/key-links для всех планов; decision coverage; четыре именованных поведенческих теста; проверка существующего release binary на crash strings; отсутствие изменений старой schema.sql; скан незавершённых реализаций; fingerprint и status validator.
- **E — результаты выполнения 02-10:** исполнитель подтвердил непосредственно наблюдённые tool responses, отдельно от SUMMARY: session 23324 — final suite 319/319, coverage, lint/build/string guard; session 1495 — live exit 100 и HTTP 403; session 37493 — два дополнительных локальных isolation controls. Файлы сырых логов final gates не сохранялись. Проверяющий не выдаёт E за собственный повторный запуск.
- Полный suite содержит регрессии Phase 1 и все 22 crash-теста; повтор целого suite без новых изменений или проблемы не выполнялся. 02-REVIEW сообщает `clean`, 171 файл, 0 findings; это отдельное свидетельство анализа кода, не замена тестам.
- `*-RED.json` — доказательства прошлых RED-итераций. Они не используются как успешный final gate.

## Достижение цели

11 критериев ROADMAP и 76 PLAN truths сведены в 24 группы без сокращения области. `VERIFIED` ниже означает реализацию и поведенческое доказательство в локальной тестовой среде (V/E), а не выполненный ручной сценарий или доступный Anthropic account.

### Observable Truths

| ID | Наблюдаемое свойство | Статус | Проверенные доказательства |
|---|---|---|---|
| T01 | Killing diffmate at any step and starting it again leaves no duplicate runs, no orphaned worktrees, and no stale worktree metadata, and interrupted runs show as `failed (interrupted: crash)`. | VERIFIED | `review/acquire.rs` повторяет только manual request, который сам восстановил свой crashed attachment; старый failed job сохраняется. `tests/crash/matrix.rs` и `matrix/assertions.rs` проверяют точное число jobs/runs и head; `helpers::assert_clean` проверяет running rows, один run на manual job, пустые run dirs и реальный `git worktree list`. Все crash cases входят в E. |
| T02 | Pushing to the PR during a run makes the draft stale. | VERIFIED | `runner.rs` сохраняет head старта, `runner/run.rs` перечитывает head после успешного агента; `render.rs` сравнивает reviewed/known head. `push_during_run_makes_the_draft_stale` удерживает реальный subprocess FIFO, меняет локальный PR и gh-ответ, проверяет первую строку, SQLite и одинаковый вывод attached review. |
| T03 | Advancing the base branch, without a push to the PR, does not make the draft stale. | VERIFIED | Сравнение использует только head. `base_move_during_run_is_not_stale` действительно продвигает base, затем проверяет отсутствие stale, reuse, один run и один agent call; `a_moved_base_branch_keeps_the_draft_current` проверяет status. |
| T04 | Running `review` twice on the same head reuses the run unless `--force` is given. | VERIFIED | Lookup `succeeded_run_for_head` происходит перед insert run/agent. `tests/dedup.rs` сравнивает сохранённый вывод, result_run_id, число вызовов агента и прозрачное reuse-сообщение; force-тест сохраняет старый draft и создаёт второй. |
| T05 | An invalid `previous_finding_id` is nulled with a warning and the run still succeeds. | VERIFIED | `normalize.rs` сравнивает с IDs из previous context. `tests/review_output/previous.rs` использует реальный ID другого target и неизвестный ID того же target; проверяет NULL, succeeded и точный warning в выводе и SQLite. |
| T06 | A PR whose review crashes diffmate every time is retried only within the retry budget, across restarts, and then reported as failed. | VERIFIED | `after_failure` и `apply_job_next` сохраняют count/max/deadline; `retry_revision` читает первую revision. `crash_every_time_exhausts_the_budget_across_watch_restarts` проверяет ровно три failed runs, общий head/base, exhausted state и выполнение другого job. Unit budget/property тесты проверяют реальные 60/300 s значения. |
| T07 | Recovery after a stop whose cause was stored but whose cleanup did not finish keeps the cause: `shutdown` re-queues an automatic job without using the budget, `user_cancel` ends the job without a retry. | VERIFIED | `store/recovery.rs` разрешает runs/jobs в одной IMMEDIATE transaction. `tests/crash/process.rs` убивает после `stop-stored`; `stop_stored.rs` проверяет разные последующие состояния. V повторно прошёл `stored_causes_survive_recovery_and_a_second_recovery_is_inert`, включая timeout и повторную recovery. |
| T08 | Stopping `watch` with SIGTERM during an automatic run records `failed (interrupted: shutdown)`, removes the worktree, and re-queues the job without using the retry budget. | VERIFIED | `watch/run.rs` вызывает общий signal wrapper с Shutdown и выходит до следующего job. `tests/watch/shutdown.rs` проверяет SIGTERM/SIGINT, повторный SIGTERM, неизменный count, worktree/metadata, процессы агента/потомка и успешный restart. |
| T09 | Ctrl-C on a `review` that acts as the runner stops and waits for the agent, records `failed (interrupted: user_cancel)` (never `agent_error`), leaves no agent or `rg` processes in its group, removes the worktree, and says so before exiting. | VERIFIED | `review/signals.rs` сохраняет причину перед cancel; `agent/stop.rs` посылает TERM/KILL текущей child group и ждёт child. `tests/signals/helpers.rs` проверяет SQL, exit 1, сообщение, отсутствие агента и дочернего stand-in, worktree и metadata. Реальный Claude/rg остаётся UAT-01. |
| T10 | Without `watch`, a `review` that attached to a `tui` re-run executes it, and does not start orphaned or automatic jobs. | VERIFIED | `review.rs` запускает только свой EnqueuedJob; hidden enqueue — согласованный stand-in TUI. `review_runs_only_the_rerun_it_attached_to` проверяет manual A, automatic B и orphaned manual C: только A получает run. |
| T11 | Two `review` commands for the same URL produce one run, a `review` started while a `tui` re-run is queued attaches to it, and `review --force` during a running non-forced job produces a second run. | VERIFIED | `store/jobs/attach.rs` + `active.rs`: один transaction, queued-first, force upgrade/exception. `tests/attach.rs`, `attach/forced.rs`, `rerun.rs` проверяют jobs, runs, force, agent calls и вывод всех waiters. |
| T12 | Timeout охватывает подготовку и агента после создания run; причина сохраняется до cancel, результат — timeout, группа и worktree очищаются. | VERIFIED | `agent_run.rs` вызывает `with_deadline`; `persist_timeout` повторяет неудачную запись до durable cause. V: `a_run_over_the_timeout_is_failed_timeout` проверил ровно 60 s paused time, failed/timeout/NULL cause и удаление путей. E включает preparation, early completion, ordering, persistence faults и shared process-group tests. |
| T13 | Config читается один раз до DB для review/watch; отсутствие/частичный файл даёт SPEC defaults; неверные keys/types/ranges — явная ошибка; status/debug не читают config. | VERIFIED | `main.rs` читает до runtime и dispatch; `config/raw.rs` deny_unknown_fields и Spanned validation. `tests/config.rs` проверяет no DB/no gh/no claude и одну безопасную error line; config unit/proptests проверяют все четыре ключа и defaults 15/60/60/2; watch invalid-config test закрывает второй entry point. |
| T14 | Миграция сохраняет Phase 1 значения и валидный приватный backup до изменения схемы. | VERIFIED | `migrate.rs` сериализует upgrade IMMEDIATE transaction; `backup.rs` VACUUM INTO через отдельное read-only соединение, validate/sync/hard-link publish. `tests/migration.rs` сравнивает каждое прежнее значение пяти таблиц и backup; WAL/concurrency/interruption/rollback tests проверяют сбои. Реальный файл владельца — UAT-10. |
| T15 | Schema v2 добавляет ровно согласованные поля/defaults/constraints; fresh=upgraded; повтор inert, newer refused, read-only v1 не изменяется. | VERIFIED | Девять ADD COLUMN и jobs_state в `migrate_v2.sql`; `src/store/schema.sql` неизменён относительно `4c8cf7d`. `store/migrate/tests.rs` сравнивает sqlite_master, содержимое/bytes, constraints; migration status test проверяет bytes, rows, отсутствие sidecars и hint. |
| T16 | Первый stop cause выигрывает, до run тоже сохраняется; уже завершённый результат не переписывается сигналом; SQLite fault не допускает unsafe cancel. | VERIFIED | Conditional UPDATE `stop_reason IS NULL`, biased completed-job branch; persisted reason используется cancelled outcome. V проверил injected SQLite trigger до успешной записи; E покрывает two-signal, pre-run, success/failure preservation, ready outer-stop и timeout-race cases. |
| T17 | Status read-only: один live head read для target с draft, fallback отмечает возраст/ошибку, target без draft не делает запрос; retry/time labels отражают SQLite. | VERIFIED | `Store::open_read_only`, `target_statuses`, `freshness::describe`, `budget::describe`. `tests/status_freshness*` и `status/tests/*` проверяют точные rows, request count, unchanged DB, stored fallback/latest-successful draft, unknown и границы времени. |
| T18 | Retry maximum фиксируется при первом retryable failure; zero budget, delays, due order и idempotent recovery соблюдаются; automatic retry сохраняет первую revision. | VERIFIED | Shared `after_failure` используется live и recovery; run/job transition коммитится вместе. `queue_selects_only_due_reviews_in_id_order`, budget/causes/orphan/property tests проверяют значения и reopen. V подтвердил все четыре поля pinned revision при новом GitHub head; manual/fresh paths в том же тесте используют новую revision. |
| T19 | Waiters следят за result_run_id без lock; stopping wait не отменяет job; attach сохраняет origin/tracking; future retry ждёт без удержания lock. | VERIFIED | `review/wait.rs`, `acquire.rs` проверяют due до lock и после recovery; stored result проверяется до acquire. `attach/finished.rs`, `rerun/delayed.rs`, `rerun/cancel.rs`, paused wait test и enqueue property доказывают эти переходы. Force intent self-recovered crash проверен `crash/restart.rs`; durable manual stop не создаёт replacement (`crash/manual_stop.rs`). |
| T20 | Неудачный post-agent recheck сохраняет успешный draft, start head и warning; fresh/reused/attached результаты получают ту же freshness. | VERIFIED | `runner/run.rs` добавляет warning, `outcome/succeeded.rs` возвращает committed stored result; `stored_result` читает current known_head. `failed_recheck_keeps_the_run`, stale attached tracer, pinned reuse и status fallback проверяют SQL и строки. Push после последнего head check виден при следующем status/run, как прямо задано планом. |
| T21 | Watch ждёт lock, восстанавливается, исполняет due queue по ID, пишет UTC events без findings; idle/wait shutdown не меняет jobs. | VERIFIED | `watch.rs`, `watch/run.rs`, `watch/wait.rs`, `lock/role.rs` подключены из main; waits используют tokio time. `tests/watch*`, UTC property/event tests и lock-role tests проверяют ожидание, live holder, shutdown, counts и отсутствие finding body. |
| T22 | Debug enqueue скрыт от help и выполняет только общую enqueue/attach операцию, без config/GitHub/lock/run/tracking mutation. | VERIFIED | `debug.rs` содержит только URL parse, Store open и enqueue; `tests/watch/debug.rs` проверяет работающий чужой lock, сломанный config, пустые fake calls, сохранённый origin/tracking и отсутствие runs; help test проверяет hidden command. |
| T23 | Все 15 crash points ограничены test feature; обычный release их не содержит; normal test/hooks/CI coverage включают matrix. | VERIFIED | `src/crash.rs` cfg на invocation и implementation; Cargo required-features, mise test/coverage `--features crash-points`, CI Ubuntu/macOS вызывает эти задачи. V проверил отсутствие обоих control strings в существующем release binary. E: release build exit 0 и 22 crash tests passed; CI wiring проверено, remote CI здесь не запускался. |
| T24 | Recovery чистит только собственные run paths и metadata собственных clones, сообщает неизвестные entries и оставляет их. | VERIFIED | `recovery/scan.rs` не следует symlink directory entries; `cleanup.rs` строит пути через Paths. `a_running_run_and_job_are_failed_and_their_leftovers_removed` проверяет run 77 без DB row, сохранение notes, warning и точные remove/prune calls; crash helpers проверяют реальные worktree metadata. |

**Score: 24/24 merged observable truths.** Непроверенных локальных transitions по отсутствию поведенческого теста не выявлено (`behavior_unverified: 0`). Все 11 roadmap truths включены в T01–T11; дополнительные T12–T24 сохраняют детали планов. Незавершённые human/live проверки перечислены отдельно и не разрешают `passed`.

### Полнота PLAN must-haves

Нумерация в строке — порядок truth в frontmatter соответствующего плана.

| План | Truths | Отображение в объединённые группы |
|---|---:|---|
| 02-01 | 6 | 1→T14; 2–6→T15 |
| 02-02 | 5 | 1–5→T13 |
| 02-03 | 6 | 1→T09; 2–5→T16; 6→T19 (ожидание окончательно реализовано в 02-06/09) |
| 02-04 | 7 | 1→T02/T17; 2–3→T17; 4→T03; 5→T17/T18; 6–7→T17 |
| 02-05 | 10 | 1–2→T07/T18; 3–8→T18; 9→T08/T18; 10→JUDGMENT-01 |
| 02-06 | 12 | 1–2→T04; 3–6→T11; 7–8→T19; 9→T05; 10–12→T19 |
| 02-07 | 7 | 1–2→T02; 3→T03; 4–7→T20 |
| 02-08 | 6 | 1–3→T12; 4→T13; 5→T16; 6→T18 |
| 02-09 | 10 | 1–3→T21; 4–6→T08/T21; 7→T22; 8→T10; 9–10→T19 |
| 02-10 | 7 | 1–2→T23; 3–4→T01; 5→T06; 6→T07; 7→T23 |

Truth 02-05.10 о PID/PGID технически подтверждён прямой трассировкой, но совпадает с judgment-tier prohibition 02-03; окончательное подтверждение этого запрета остаётся human item. Дополнительный положительный truth не скрывает этот флаг.

Уточнения относительно первоначального текста планов проверены по SPEC: same-head retry сделан уже в Phase 2; stop cause при ошибке записи сохраняется до cancel; один run на job применяется к manual jobs, automatic retries намеренно дают несколько; повторный manual request после собственного crash recovery создаёт новый job, не оживляет failed job и сохраняет force. Для failed run используется канонический cause text без старой оболочки `run N failed (...)`. Это не ослабляет требуемый исход.

### Required Artifacts

CLI `verify.artifacts` проверил 34 declarations: 29 pattern checks passed, 5 pattern misses. Все пять разрешены чтением подключённых подмодулей; missing/stub/orphaned артефактов нет.

| Артефакты | Содержательность и подключение | Статус |
|---|---|---|
| 02-01: migrate.rs, migrate_v2.sql, tests/migration.rs | Store::open вызывает миграцию; SQL и проверки данных реальные | VERIFIED 3/3 |
| 02-02: config.rs, raw.rs, tests/config.rs | Startup load и отрицательные тесты до DB | VERIFIED 3/3 |
| 02-03: cancel.rs, review/signals.rs, tests/signals.rs | Typed causes, durable write, subprocess assertions | VERIFIED 3/3 |
| 02-04: status/freshness.rs, budget.rs, when.rs, tests/status_freshness.rs | Read model питается gh/SQLite; time/budget formatter вызваны status | VERIFIED 4/4 |
| 02-05: recovery/rules.rs, store/retry.rs, recovery/tests.rs | `next_runnable_job` вынесен в подключённый `store/retry/queue.rs`; budget test — в `recovery/tests/budget.rs` | VERIFIED 3/3 |
| 02-06: jobs/attach.rs, store/result.rs, review/wait.rs, tests/attach.rs, tests/dedup.rs | Транзакция, реальные SELECT, CLI wait и повтор | VERIFIED 5/5 |
| 02-07: store/targets.rs, tests/stale_review.rs | Оба runner head-write call sites найдены, tracer меняет PR в удержанном run | VERIFIED 2/2 |
| 02-08: agent_run/deadline.rs, agent_run/tests.rs | Production deadline и paused-time SQL/filesystem assertions | VERIFIED 2/2 |
| 02-09: watch/run.rs, watch/events.rs, debug.rs, tests/watch.rs, tests/rerun.rs | `utc` в `events/date.rs`, SIGTERM test в подключённом `tests/watch/shutdown.rs` | VERIFIED 5/5 |
| 02-10: crash.rs, crash/matrix.rs, crash/budget.rs, crash/stop_stored.rs | `stop-stored` строка находится в общем `tests/crash/process.rs::crash_after_stop`, который вызывают оба cause-теста | VERIFIED 4/4 |

### Key Link Verification

CLI `verify.key-links`: **23/23**. Дополнительно вручную прослежены вызовы после разбиения файлов:

| Цепочка | Проверенное соединение | Статус |
|---|---|---|
| main → config → review/watch | Settings загружается перед runtime и перед enqueue/Store::open | WIRED |
| review/debug → Store enqueue → active selection | IMMEDIATE transaction охватывает target и attach/create, request force не теряется | WIRED |
| lock → recovery → execute_review_job | Recovery вызывается после OS lock; review ограничен выбранным job, watch получает due job | WIRED |
| signals/deadline → job stop_reason → cancel → child stop | Успех durable write предшествует channel notification; process group берётся из локального Child | WIRED |
| cancelled/failed outcome и recovery → after_failure → apply_job_next | Одни правила budget/cause; failed run/job атомарны | WIRED |
| resolve → retry_revision → succeeded_run_for_head | Retry сначала выбирает сохранённую revision; reuse сравнивает именно выбранный head | WIRED |
| head read → targets → stored_result/status → render | Источник head и check time — gh и SQLite, не константа | WIRED |
| mise tasks → crash feature → integration target → CI matrix | Normal test, coverage и hooks включают crash suite; default build исключает feature | WIRED |

### Data-Flow Trace (Level 4)

| Вывод | Источник и преобразование | Статус |
|---|---|---|
| Findings fresh/reused/attached review | Валидированный agent output → normalize → finish_succeeded transaction → drafts/findings → stored_result → renderer | FLOWING |
| Первая stale строка review | Реальный gh start/post-agent head → targets.head_sha → StoredJob.known_head; сравнение с runs.head_sha | FLOWING |
| Status freshness | Latest successful draft head из SQL + один live gh head; при ошибке сохранённые head/time и явный failed marker | FLOWING |
| Retry count / due / exhausted | jobs.retry_count/max_retries/next_attempt_at и failed-run existence → RetryView → budget formatter | FLOWING |
| Watch events | Реальный ReviewResult/JobEnd + текущий timestamp → counts/requeue/retry lines | FLOWING |

### Prohibitions

13 test-tier запретов имеют wired negative assertions. Два judgment-tier запрета ниже остаются **unverified-prohibition — human review recommended**, даже при положительном анализе исходников.

| План / запрет | Enforcement evidence | Статус |
|---|---|---|
| 02-01 — не терять старые значения | Snapshot всех старых колонок пяти таблиц и backup, rollback/refusal tests | VERIFIED |
| 02-02 — не игнорировать неверный config | Invalid/unknown/type/range tests; no DB/no CLI calls; single-line errors | VERIFIED |
| 02-03 — не сигналить чужой или сохранённый в DB PID/PGID | `supervise` берёт `child.id()`; recovery/scan/cleanup не вызывают signaller | UNCERTAIN (judgment; JUDGMENT-01) |
| 02-04 — не показывать неверную current/fresh отметку | Exact stale/current/fallback/unknown assertions, head-only tests | VERIFIED |
| 02-04 — status не мутирует и не запускает recovery | Read-only connection, unchanged snapshot, gh pr-view route assertions | VERIFIED |
| 02-05 — user_cancel никогда не retry | Causes/terminal table плюс real-process stop-stored tests | VERIFIED |
| 02-05 — budget bounded и recovery не сигналит DB IDs | Failure-sequence property, reopen budget, exact fake Git calls и отсутствующий process seam у recovery; human confirmation границы сигналов сохранён отдельно | VERIFIED (test evidence; judgment cross-reference выше) |
| 02-06 — reuse прозрачен, forced не reuse | Exact stderr и agent/run/draft counts в dedup/force tests | VERIFIED |
| 02-06 — attach/manual enqueue не меняет tracking/origin | Enqueue sequence property и debug/delayed attachment tests | VERIFIED |
| 02-07 — stale warning обязателен при несовпадении | First-line equality fresh/attached и stored pinned reuse tests | VERIFIED |
| 02-08 — timeout не success/no_findings/agent_error | Exact run/job failure_kind, no cause и deadline assertions | VERIFIED |
| 02-09 — debug не publish/GitHub/lock/run/tracking | `enqueue_ignores_config_and_lock_and_attaches_without_running` | VERIFIED |
| 02-09 — watch не poll/producer в этой фазе | Watch читает queued jobs; enqueue вызова и отдельного gh polling пути нет | UNCERTAIN (judgment; JUDGMENT-02) |
| 02-10 — release не содержит crash-point machinery | cfg boundary + fresh binary string inspection + E release build | VERIFIED |
| 02-10 — cleanup не выходит за own paths, unknown сохраняются | Paths-only scanner, notes-preservation/exact-call test и реальные metadata assertions | VERIFIED |

### Requirements Coverage

Все REL-01–REL-13 заявлены в PLAN frontmatter; orphaned IDs: **0**. `SATISFIED (automated)` — покрытие реализации, не команда отметить фазу или незавершённые требования Complete.

| Requirement | Source plans | Состояние реализации | Доказательство / остающийся acceptance |
|---|---|---|---|
| REL-01 | 02-10 | SATISFIED (automated) | T01/T23/T24; UAT-09 и release live gate открыты |
| REL-02 | 02-04, 02-07 | SATISFIED (automated) | T02/T17/T20; UAT-04 открыт |
| REL-03 | 02-04, 02-07 | SATISFIED (automated) | T03; UAT-02 открыт |
| REL-04 | 02-06 | SATISFIED (automated) | T04; UAT-03 открыт |
| REL-05 | 02-06 | SATISFIED (automated) | T05, real ID/unknown ID regression |
| REL-06 | 02-01, 02-04, 02-05, 02-10 | SATISFIED (automated) | T06/T18; UAT-08 открыт |
| REL-07 | 02-01, 02-03, 02-05, 02-10 | SATISFIED (automated) | T07/T14/T16; связанные owner/live checks открыты |
| REL-08 | 02-09 | SATISFIED (automated) | T08/T21; UAT-06 открыт |
| REL-09 | 02-03 | SATISFIED (automated) | T09/T16; UAT-01 открыт |
| REL-10 | 02-09 | SATISFIED (automated) | T10/T19/T22; UAT-07 открыт |
| REL-11 | 02-06, 02-09 | SATISFIED (automated) | T11/T19; UAT-03 открыт |
| REL-12 | 02-08 | SATISFIED (automated) | T12; UAT-05 открыт |
| REL-13 | 02-02, 02-08, 02-09 | SATISFIED (automated) | T13, defaults/ranges, both startup commands, timeout and persisted maximum |

### Decision Coverage

CLI `check.decision-coverage-verify` завершился exit 0: **16/16** trackable решений учтены, `not_honored: []`, `blocking: false`.

Возвращённое сообщение: “All trackable CONTEXT.md decisions are honored by shipped artifacts.”

Ручная сверка подтвердила D-01–03 runner-only watch/debug/events, D-04–07 freshness, D-08–12 config, D-13–16 crash feature/test scope и сохранённые UAT требования. Heuristic gate не доказывает выполнение отложенных hand checks.

## Проверки

### Behavioral Spot-Checks

V запустил ровно четыре выбранных теста одним фильтром:

```text
NEXTEST_TEST_THREADS=2 mise run --force test -- -E 'test(a_failed_signal_write_does_not_cancel_until_persisted) | test(a_run_over_the_timeout_is_failed_timeout) | test(stored_causes_survive_recovery_and_a_second_recovery_is_inert) | test(automatic_retries_keep_the_stored_revision_when_the_pr_head_moves)'
```

Итоговый Nextest run `1160e80b-bca9-444a-ba73-0d4fa107cc39`: **4 passed, 326 excluded by filter**, 0.392 s, command exit 0. В первом sandboxed запуске те же 4 tests прошли, но mise завершился exit 1 из-за записи служебного cache; повтор с разрешённой эскалацией завершился успешно. Исключённые фильтром 326 — не новые skipped tests.

| Проверка | Происхождение | Результат |
|---|---|---|
| Stop cause write fault не допускает cancel | V, названный тест выше | PASS |
| Timeout точно 60 s, правильный SQL outcome и cleanup | V, названный тест выше | PASS |
| Recovery сохраняет shutdown/user_cancel/timeout и idempotence | V, названный тест выше | PASS |
| Новый GitHub head не подменяет pinned retry revision | V, названный тест выше | PASS |
| 34 artifacts, 23 links | V, CLI + ручная проверка 5 перемещений | PASS после разрешения pattern misses |
| Decision coverage | V | 16/16, exit 0 |
| Release crash strings | V, bytes `target/release/diffmate` | Обе строки отсутствуют |
| Final `mise run --force test` | E, session 23324 | 319 passed, 11 ignored live checks, exit 0; 22 crash tests входят в 319 |
| Final lint / build / crash-string guard | E, session 23324 и 02-10 handoff | Exit 0 |
| `mise run coverage` | E, session 23324 | 94.80% line coverage: 4594 lines, 239 missed; gate 80%, exit 0 |
| deps:check / vuln / docs:check | E, 02-10 observed handoff | Exit 0; non-fatal existing duplicate-version warnings в vuln |
| Полный `mise run test:e2e` | E, session 1495 | **FAILED/externally blocked**, exit 100: 4 passed, auth failed, 6 not run по fail-fast |
| Отдельные rev07/rev08 local controls | E, session 37493 | 2 passed, exit 0, без нового model call |
| Совокупный live итог | E | 6 non-model checks passed; 1 auth failed; 4 model-backed checks не запускались |

Никакой формулировки «все gates пройдены» из этих данных не следует. Полный suite, build, lint, coverage, deps, vuln, live e2e и remote CI в ходе V повторно не запускались. Реальная база/конфигурация владельца не изменялись.

### Probe Execution

N/A: самостоятельных `probe-*.sh` в плане фазы не объявлено. Claude flag/version probes — часть продукта и существующих тестов; их live результаты отражены в E. Они не подменяют заблокированную auth/model проверку.

### Test Quality Audit

| Набор | Linked requirements | Active / skipped | Уровень assertions | Вывод |
|---|---|---|---|---|
| migration + migrate tests | REL-07 prerequisite | Active; 0 disabled | Старые значения/bytes, backup, constraints, rollback/concurrency | Данные зафиксированы до upgrade, не сгенерированы его результатом |
| config unit/proptests + CLI | REL-13 | Active; 0 disabled | Точные defaults/ranges/errors и отсутствие side effects | Достаточно |
| crash matrix/restart/budget/stop | REL-01/06/07 | 22 active; required feature включён в normal test | SIGKILL/restart, exact SQL counts/causes/head, filesystem/metadata | Достаточно для согласованных 15 checkpoints |
| signals/watch/persistence | REL-08/09 | Active; 0 disabled | Process disappearance, DB causes, requeue, restart | Проверка `ps` не принимает ошибку инспекции за отсутствие процесса |
| stale/status/dedup/attach/rerun/previous | REL-02–05/10/11 | Active; 0 disabled | Multi-step CLI, exact output, call counts, stored values | Достаточно; реальные dependency seams, не фиктивный rendering |
| timeout/retry/queue/property | REL-06/07/12/13 | Active; 0 disabled | Paused time, injected SQLite faults, budget sequences, SQL boundaries | Достаточно для локальных переходов |
| isolation/live model | Regression Phase 1 / final live gate | 11 intentionally ignored normal-suite cases | External controls и model assertions | Отдельный gate; auth failed и 4 unrun сохранены |

Circular expected-output generators, незадействованные требуемые тесты и assertions только на existence вместо требуемого поведения не обнаружены. Snapshot миграции снимается со старой schema до изменения; expected states и deadlines независимы от output нового кода.

**Проверка против собственной гипотезы о завершённости:** real-Claude acceptance остаётся частично непроверенным из-за auth; crash budget integration тест принудительно переводит `next_attempt_at` в due и поэтому сам не доказывает ожидание 1/5 минут — его дополняют pure/paused-time delay и due-boundary тесты; real-Claude timeout/process behavior покрыто fake-process path, но остаётся UAT-05. Эти ограничения не скрыты под общим счётчиком 319.

### Anti-Patterns Found

Незавершённых `TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER/todo!/unimplemented!` в проверенных src/tests/scripts не найдено. Пустые значения являются defaults/negative outcomes и наполняются реальными query/result paths. Новых implementation blockers: **0**.

При постоянной недоступности SQLite durable-first остановка не сможет завершиться безопасно до сохранения cause: retry записи продолжается, owned future сохраняется. Это явно описанный operational limit, не бесконечный retry самого review и не заявленная гарантия успешной остановки при недоступной БД. SPEC не изменён.

## Human Verification Required

Frontmatter содержит **14** структурированных пунктов: **10 hand checks**, **2 внешних live пункта**, **2 judgment confirmations**. Это единый список для следующего `02-UAT.md`; отдельного параллельного плана не создавалось.

| ID | Действие и ожидаемый результат | Почему остаётся открытым |
|---|---|---|
| UAT-01 | Ctrl-C реального review: user_cancel, сообщение, no child/group leftovers | WINDOWS 1 |
| UAT-02 | Продвижение только base: draft current | WINDOWS 2 |
| UAT-03 | Reuse, force и параллельные review: правильные counts и сообщения | WINDOWS 3 |
| UAT-04 | Реальный push во время review: stale first line/status | WINDOWS 4 |
| UAT-05 | Реальный минутный timeout: failed timeout и cleanup | WINDOWS 5 |
| UAT-06 | SIGTERM watch и restart: shutdown/requeue без бюджета | WINDOWS 7 |
| UAT-07 | Attached re-run без watch: чужой backlog не исполняется | WINDOWS 8 |
| UAT-08 | Три live SIGKILL automatic attempts: budget exhausted | WINDOWS 9 |
| UAT-09 | Live kill во время preparation/agent: recovery и metadata clean | WINDOWS 10 |
| UAT-10 | Реальная Phase 1 DB после backup/upgrade: старые значения сохранены | Harvested 02-01 Task 3; не был записан в WINDOWS |
| LIVE-01 | После owner-directed account recovery: auth с production flags работает | WINDOWS 11; **blocked**, HTTP 403; автоматический повтор не выполняется |
| LIVE-02 | Max-turns, clean, bug и fork model tests дают ожидаемые результаты | WINDOWS 12; четыре случая **not run** |
| JUDGMENT-01 | Подтвердить существующую границу сигнала только locally spawned child group | Judgment-tier prohibition, код соответствует |
| JUDGMENT-02 | Подтвердить runner-only watch без polling/producers | Judgment-tier prohibition, код соответствует |

WINDOWS 6 не является открытым пунктом: durable-first signal fix проверен непосредственно V и fault-injection assertions. Одобрение additive-backup и решение сохранить внешний blocker не означают выполнения UAT или принятия waiver.

## Границы фазы и regression

Phase 3 явно владеет автоматическими producers, tracking, debounce, producer-side deduplication/budget для новых heads, transient agent_error retries и уведомлениями. Phase 4 владеет TUI и publication/reconcile/sync. Их отсутствие не объявляется дефектом Phase 2. Сейчас automatic crash/timeout retries bounded и привязаны к первой записанной revision; обычные agent_error terminal, как зафиксировано в планах и передаче Phase 3.

Предыдущий 01-VERIFICATION использован для регрессионного контекста: failure ≠ no_findings, isolation, fork fetch, missing flags, status/lock и сохранённые данные. Их non-live tests входят в E final suite. Старые live результаты не доказывают текущую доступность auth: нынешний 403 и четыре unrun model checks остаются самостоятельным незакрытым regression gate.

## Gaps Summary

**Implementation gaps: 0.** Нет основания создавать code gap-closure plan или менять SPEC. **Фаза не готова к завершению:** статус `human_needed`, внешний live-gate `blocked`, перечисленные acceptance checks открыты; `passed` здесь недопустим.

Fingerprint получен только через `verification.fingerprint`: 205 covered files, включая автоматически добавленные PLAN/SUMMARY. Переданы 172 изменённых implementation/test/tooling пути canonical phase scope (включая Cargo.lock) и дополнительные прочитанные evidence files. CLI исключает сам report и инертные planning-root tracking документы; shared STATE/ROADMAP/config/runtime state не редактировались.


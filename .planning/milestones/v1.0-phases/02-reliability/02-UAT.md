---
status: complete
phase: 02-reliability
source: [02-VERIFICATION.md]
started: 2026-11-03T15:35:56.260Z
updated: 2026-11-14T18:27:19Z
---

## Current Test

[testing complete]

## Tests

### 1. UAT-01: На release binary выполнить review реального fixture PR и нажать Ctrl-C во время агента; проверить status и процессы группы.
expected: Exit 1, сообщение Run cancelled; recorded as interrupted, failed (interrupted: user_cancel), отсутствие агента и rg этой группы.
result: pass
source: WINDOWS-1

### 2. UAT-02: Продвинуть только base ветку fixture PR и выполнить status.
expected: Черновик остаётся current при неизменном PR head.
result: pass
source: WINDOWS-2

### 3. UAT-03: Дважды выполнить review одного head, затем --force; параллельно запустить два review второго PR.
expected: Повтор явно переиспользует run; --force создаёт новый; одновременные обычные запросы получают один run.
result: pass
source: WINDOWS-3

### 4. UAT-04: Во время реального review отправить commit в PR, затем прочитать результат и status.
expected: Exit 0; первая строка review и status явно говорят stale.
result: pass
source: WINDOWS-4

### 5. UAT-05: На release binary проверить run_timeout_minutes = 1 с review длительнее минуты, затем вернуть настройку.
expected: Failed (timeout), exit 1, очищенный worktree и отсутствие процессов агента этой группы.
result: pass
source: WINDOWS-5

### 6. UAT-06: Поставить automatic job через debug enqueue, запустить watch, отправить SIGTERM во время агента и перезапустить watch.
expected: Shutdown записан, worktree удалён, job requeued без расхода бюджета и затем выполнен.
result: pass
source: WINDOWS-7

### 7. UAT-07: Без watch поставить manual re-run URL_A и automatic URL_B; выполнить review URL_A.
expected: Review присоединяется к A и исполняет только A; B остаётся queued без run.
result: pass
source: WINDOWS-8

### 8. UAT-08: На release binary трижды убить watch через SIGKILL во время одного automatic review, перезапуская его между попытками.
expected: Status показывает retry 1 of 2, retry 2 of 2, затем no retries left; четвёртой попытки нет.
result: pass
source: WINDOWS-9

### 9. UAT-09: На release binary убить review во время подготовки и во время агента; после каждого сбоя повторить review.
expected: Повтор успешен, существовавший run помечен interrupted: crash, runs directory пуст и git worktree metadata содержит только bare clone.
result: pass
source: WINDOWS-10

### 10. UAT-10: После отдельной копии реальной Phase 1 базы выполнить release status, review fixture PR и status; сравнить прежние данные и v1-backup.
expected: Прежние target/job/run/draft/finding значения сохранены, backup валиден; outdated-schema hint до первого upgrade, новые данные доступны после.
result: pass
source: 02-01 Task 3 human-check

### 11. LIVE-01: На grok CLI с боевыми флагами проверить канарейку недоверенного worktree и что запуск аутентифицируется.
expected: `untrusted_worktree_canary_is_not_loaded` — `grok inspect` пишет `Project trusted: no`, не показывает `AGENTS.md` и `CLAUDE.md` worktree, claude и cursor rules стоят OFF (env), а боевой запуск доходит до модели.
result: pass
source: WINDOWS-11

### 12. LIVE-02: Четыре модельных случая на grok: лимит ходов, чистый PR, PR с багом и PR из форка.
expected: `live_max_turns_one_is_failed_not_no_findings` даёт `failed` (`agent_error`, subtype `error_max_turns`), не `no_findings`. Чистый PR даёт `no_findings`. Баг и fork дают находку на ожидаемой строке `calc.py:7`.
result: pass
source: WINDOWS-12

### 13. JUDGMENT-01: При итоговом UAT подтвердить проверенную границу сигналов: только group текущего локально порождённого agent child, никогда PID/PGID из SQLite.
expected: Сохранённые идентификаторы служат диагностикой; recovery не сигналит их.
result: pass
source: 02-03 prohibition

### 14. JUDGMENT-02: При итоговом UAT подтвердить границу watch этой фазы: потребление существующих review jobs без собственного polling/producer.
expected: Watch не создаёт jobs и не опрашивает GitHub вне исполняемого review.
result: pass
source: 02-09 prohibition

## Summary

total: 14
passed: 14
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

None. On 2026-11-14 the five Grok checks passed under `cargo nextest run --locked --features crash-points --run-ignored only --test-threads=1`: `untrusted_worktree_canary_is_not_loaded` 31.285s, `live_max_turns_one_is_failed_not_no_findings` 8.036s, `live_clean_pr_has_no_findings` 37.360s, `live_fork_pr_is_reviewed_the_same_way` 63.677s, `live_obvious_bug_is_anchored_to_line_7` 64.126s. Summary: 5 passed, exit 0. The ignored Claude isolation checks remain the slice 1 record and were not part of this run.

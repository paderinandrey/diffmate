---
status: partial
phase: 02-reliability
source: [02-VERIFICATION.md]
started: 2026-11-03T15:35:56.260Z
updated: 2026-11-03T15:35:56.260Z
---

## Current Test

[testing paused — 14 items outstanding]

Live gate blocked by Anthropic HTTP 403. Owner decision: «Сохранить live gate как заблокированный». No automatic Claude API retry or waiver. Resume with `/gsd-verify-work 02`.

## Tests

### 1. UAT-01: На release binary выполнить review реального fixture PR и нажать Ctrl-C во время агента; проверить status и процессы группы.
expected: Exit 1, сообщение Run cancelled; recorded as interrupted, failed (interrupted: user_cancel), отсутствие агента и rg этой группы.
result: [pending]
source: WINDOWS-1

### 2. UAT-02: Продвинуть только base ветку fixture PR и выполнить status.
expected: Черновик остаётся current при неизменном PR head.
result: [pending]
source: WINDOWS-2

### 3. UAT-03: Дважды выполнить review одного head, затем --force; параллельно запустить два review второго PR.
expected: Повтор явно переиспользует run; --force создаёт новый; одновременные обычные запросы получают один run.
result: [pending]
source: WINDOWS-3

### 4. UAT-04: Во время реального review отправить commit в PR, затем прочитать результат и status.
expected: Exit 0; первая строка review и status явно говорят stale.
result: [pending]
source: WINDOWS-4

### 5. UAT-05: На release binary проверить run_timeout_minutes = 1 с review длительнее минуты, затем вернуть настройку.
expected: Failed (timeout), exit 1, очищенный worktree и отсутствие процессов агента этой группы.
result: [pending]
source: WINDOWS-5

### 6. UAT-06: Поставить automatic job через debug enqueue, запустить watch, отправить SIGTERM во время агента и перезапустить watch.
expected: Shutdown записан, worktree удалён, job requeued без расхода бюджета и затем выполнен.
result: [pending]
source: WINDOWS-7

### 7. UAT-07: Без watch поставить manual re-run URL_A и automatic URL_B; выполнить review URL_A.
expected: Review присоединяется к A и исполняет только A; B остаётся queued без run.
result: [pending]
source: WINDOWS-8

### 8. UAT-08: На release binary трижды убить watch через SIGKILL во время одного automatic review, перезапуская его между попытками.
expected: Status показывает retry 1 of 2, retry 2 of 2, затем no retries left; четвёртой попытки нет.
result: [pending]
source: WINDOWS-9

### 9. UAT-09: На release binary убить review во время подготовки и во время агента; после каждого сбоя повторить review.
expected: Повтор успешен, существовавший run помечен interrupted: crash, runs directory пуст и git worktree metadata содержит только bare clone.
result: [pending]
source: WINDOWS-10

### 10. UAT-10: После отдельной копии реальной Phase 1 базы выполнить release status, review fixture PR и status; сравнить прежние данные и v1-backup.
expected: Прежние target/job/run/draft/finding значения сохранены, backup валиден; outdated-schema hint до первого upgrade, новые данные доступны после.
result: [pending]
source: 02-01 Task 3 human-check

### 11. LIVE-01: После восстановления доступа к Anthropic по отдельному указанию владельца повторить isolation_flags_keep_authentication.
expected: Настоящий Claude работает с production isolation flags, без HTTP 403.
result: blocked
blocked_by: third-party
reason: Сейчас внешний account-verification барьер. Владелец выбрал сохранить live gate заблокированным; это не waiver и не разрешение на автоматический повтор.
source: WINDOWS-11

### 12. LIVE-02: После восстановления account access выполнить четыре оставшихся model-backed случая через mise run test:e2e.
expected: Max-turns даёт failed, clean PR — no_findings, bug и fork PR — ожидаемую находку calc.py:7.
result: blocked
blocked_by: third-party
reason: Четыре проверки не запускались после fail-fast на auth; локальные fakes не доказывают модельный результат.
source: WINDOWS-12

### 13. JUDGMENT-01: При итоговом UAT подтвердить проверенную границу сигналов: только group текущего локально порождённого agent child, никогда PID/PGID из SQLite.
expected: Сохранённые идентификаторы служат диагностикой; recovery не сигналит их.
result: [pending]
source: 02-03 prohibition

### 14. JUDGMENT-02: При итоговом UAT подтвердить границу watch этой фазы: потребление существующих review jobs без собственного polling/producer.
expected: Watch не создаёт jobs и не опрашивает GitHub вне исполняемого review.
result: [pending]
source: 02-09 prohibition

## Summary

total: 14
passed: 0
issues: 0
pending: 12
skipped: 0
blocked: 2

## Gaps

None identified in implementation. External authentication and incomplete acceptance are preserved above; no code gap-closure plan is required.

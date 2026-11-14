# Phase 4: Review and publish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-11-09
**Phase:** 4-review-and-publish
**Areas discussed:** Чтение черновика, Клавиши кроме a/r/c, Файл в $EDITOR, Подтверждения

---

## Чтение черновика

| Option | Description | Selected |
|--------|-------------|----------|
| Список и текст выбранного | List rows plus the full text of the selected finding. | ✓ |
| Один finding на весь экран | Only the current finding, with a counter for the rest. | |
| You decide | Planner picks a layout that still shows every decision and the selected body. | |

**User's choice:** Список и текст выбранного
**Notes:** None.

| Option | Description | Selected |
|--------|-------------|----------|
| Два экрана | Queue on one screen, draft on the next. Enter opens, one key returns. | ✓ |
| Очередь сверху | Queue rows stay visible above the finding list. | |
| You decide | Planner picks the transition and keeps the list plus the selected body. | |

**User's choice:** Два экрана
**Notes:** Return key was fixed in the next area as Esc.

| Option | Description | Selected |
|--------|-------------|----------|
| Отдельный блок | Findings only in the list. Limitation and future improvements sit under the text. | ✓ |
| Строки того же списка | Limitation and future improvements are further list rows, without a kept or dropped decision. | |
| You decide | Planner separates notes from findings. | |

**User's choice:** Question skipped. Recommended option used after it was restated in the following question, which the owner answered.
**Notes:** Recorded as D-03.

| Option | Description | Selected |
|--------|-------------|----------|
| Все targets, новые сверху | Same targets as `status`, newest first, including an untracked target that still has a publication. | ✓ |
| Как status, по id | Same rows and the same `ORDER BY t.id` order as `diffmate status`. | |
| Только живые черновики | Only tracked targets with a draft or an open publication. | |

**User's choice:** Все targets, новые сверху
**Notes:** Implemented as target id descending, the reverse of `status`.

**Continue check:** К клавишам.

---

## Клавиши кроме a/r/c

| Option | Description | Selected |
|--------|-------------|----------|
| k, d, e | `k` kept, `d` dropped, `e` opens `$EDITOR`. | ✓ |
| Пробел переключает | Space cycles undecided, kept, dropped. `e` still edits. | |
| You decide | Planner picks decision keys that avoid `a`, `r`, and `c`. | |

**User's choice:** k, d, e
**Notes:** `a`, `r`, and `c` stay the SPEC.md publish events.

| Option | Description | Selected |
|--------|-------------|----------|
| Стрелки, Enter, Esc | Arrows move. Enter opens the draft. Esc returns. | ✓ |
| j и p, Enter, Esc | `j` down, `p` up, Enter and Esc as above. | |
| You decide | Planner picks movement that leaves the action keys free. | |

**User's choice:** Стрелки, Enter, Esc
**Notes:** None.

| Option | Description | Selected |
|--------|-------------|----------|
| Схема K o n i x y b q | `K` keep remaining undecided, `o` browser, `n` re-run, `i` limitation, `x` discard, `y` reconcile, `b` abandon, `q` quit. | ✓ |
| Только K и q | Lock `K` and `q`. Planner assigns the other keys. | |
| You decide | Planner assigns every remaining key. | |

**User's choice:** Question skipped. Recommended map used.
**Notes:** Recorded as D-07.

| Option | Description | Selected |
|--------|-------------|----------|
| Edited только при изменении | `edited` only when the saved text differs. | ✓ |
| Любое сохранение ставит edited | Any successful save sets `edited`. | |
| You decide | Planner binds `e` to `edited` and does not publish an empty finding. | |

**User's choice:** Question skipped. Recommended rule used.
**Notes:** Recorded as D-08.

**Continue check:** Question skipped. Moved on to the editor file.

---

## Файл в $EDITOR

| Option | Description | Selected |
|--------|-------------|----------|
| Один выбранный текст | One temporary file for the selected finding or one future improvement. | ✓ |
| Весь черновик одним файлом | One file for the whole draft, parsed back into sections on save. | |
| You decide | Planner picks a file that keeps one finding's edit off the others. | |

**User's choice:** Question skipped. Recommended option used.
**Notes:** Recorded as D-09. The owner then skipped the ready check. Confirmations were not asked as their own four questions.

---

## Подтверждения

No options were presented. The ready check named the recommended rule and was skipped.

**User's choice:** Recommended rule used.
**Notes:** Recorded as D-10 and D-11. A current decided draft publishes from `a`, `r`, or `c`. Stale, head-moved, discard, and abandon highlight the refusal.

---

## Claude's Discretion

- Finding file layout, empty save, and a missing `EDITOR`.
- Placement of the previous-decision hint on the row.
- Browser open command, comment-sync interval, and prompt wording.

## Deferred Ideas

- Phase 2 UAT remains open.
- Phase 3 is not accepted from this discussion.
- Phase 5 remains launchd and `doctor`.

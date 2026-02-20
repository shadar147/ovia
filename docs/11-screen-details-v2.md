# Ovia — Screen Details v2 (Laptop-first)

Ниже максимально приземлённо: что именно пользователь видит, какие поля за ними стоят и что можно нажать.

---

## A. Dashboard (Exec)

### 1) KPI cards (верхний ряд)
1. **Delivery Health (0-100)**
   - Формула (пример MVP):
     - 30% sprint completion rate
     - 25% blocker aging score
     - 20% MR review latency score
     - 15% pipeline success score
     - 10% carry-over (spillover) score
   - Клик: открывает breakdown модалку.

2. **Release Risk (Low/Med/High + 0-100)**
   - Факторы:
     - блокеры > X дней
     - открытые high-priority баги
     - failing pipelines на release branch
     - доля stale MR
   - Клик: список рисков с владельцами.

3. **Throughput**
   - Закрытые Jira issues за период
   - Разбивка: bug/feature/chore

4. **Review Latency**
   - Медиана времени от MR open до first review
   - P90 отдельно в tooltip

### 2) Блок “Trends”
- Графики:
  - velocity by sprint
  - cycle time trend (median + P75)
  - lead time trend
- Фильтры:
  - product, team, date range
- Drilldown:
  - click point -> Team/Issues list

### 3) Блок “Top Risks & Blockers”
- Таблица (top 10):
  - entity type (issue/MR/pipeline)
  - title
  - owner (canonical person)
  - age
  - impact scope
  - status
  - source link
- Actions:
  - “assign owner” (phase 2)
  - “send to report”

### 4) Narrative Insight
- Авто-текст:
  - “что изменилось за неделю”
  - “почему вероятно изменилось”
  - “что проверить дальше”
- Обязательные citations:
  - min 2 ссылки на источники

---

## B. Team → Identity Mapping

### 1) Таблица полей
- Canonical person
  - `person_id`, display_name, primary_email
- GitLab
  - `gitlab_user_id`, username, email, last_active_at
- Jira
  - `jira_account_id`, displayName, email, last_seen
- Confluence
  - `confluence_account_id`, displayName, email
- Match status
  - `auto | verified | conflict | unmatched | ignored`
- Confidence
  - `0.00 .. 1.00`
- Actions
  - confirm / remap / split / ignore

### 2) Match rationale drawer
Показывает:
- какие правила сработали:
  - exact email match (+0.7)
  - username similarity (+0.2)
  - display name similarity (+0.1)
- альтернативные кандидаты
- история изменений

### 3) Bulk операции
- Confirm all `confidence >= threshold` and `status=auto`
- Export unresolved conflicts CSV

### 4) KPI для качества матчинга
- Mapping coverage %
- Conflict rate %
- Verified rate %

---

## C. Team → Person 360

### Header
- Canonical profile card
- role/team tags
- mapped accounts chips

### Metrics row
- Jira: created/closed issues, blockers, reopened rate
- GitLab: commits, MRs opened/merged, avg review wait
- Confluence: pages updated, comments, key docs touched

### Timeline
События в единой ленте:
- issue transitioned
- MR opened/reviewed/merged
- pipeline failed/passed
- confluence page updated

Фильтры:
- source, event type, severity

### Insights
- “high code activity, low jira movement”
- “high blocker ownership concentration”
- “review bottleneck contributor”

---

## D. Ask Ovia

### Input area
- free-form query
- filters: product/team/date/source
- template prompts:
  - “Что блокирует релиз?”
  - “Почему просел velocity?”

### Output contract
- **Answer** (чётко и коротко)
- **Assumptions**
- **Confidence** (high/med/low)
- **Evidence**
  - список citations с deep links
  - excerpt/snippet

### Modes
- `summary mode` — управленческий ответ
- `analyst mode` — с формулами/SQL-логикой

---

## E. Reports

### Templates
- Weekly Executive
- Team Delivery Digest
- Release Readiness
- Identity Mapping Quality

### Schedule
- cron-like setup
- timezone-aware
- channel routing: telegram/slack/email

### Run history
- started_at, duration, status
- failure_reason + retry

---

## F. Settings

### Connections
- Jira/GitLab/Confluence creds status
- last sync time
- test connection button

### Sync policy
- full sync toggle (admin only)
- incremental interval
- retention rules

### Roles
- admin / manager / analyst / viewer
- source-level access constraints

---

## Canonical API sketch (MVP)

- `GET /dashboard/kpis`
- `GET /dashboard/risks`
- `GET /team/identity-mappings`
- `POST /team/identity-mappings/confirm`
- `POST /team/identity-mappings/remap`
- `GET /team/person/:personId`
- `POST /ask`
- `GET /reports/templates`
- `POST /reports/run`

---

## Data model sketch (identity core)

### `identity_profiles`
- id (uuid)
- org_id
- display_name
- primary_email
- team
- role
- created_at/updated_at

### `identity_accounts`
- id (uuid)
- profile_id (nullable before match)
- source (`gitlab|jira|confluence`)
- external_id
- username
- email
- display_name
- raw_ref

### `identity_links`
- id
- profile_id
- account_id
- status
- confidence
- rule_trace (jsonb)
- verified_by
- verified_at

### `identity_events`
- id
- link_id
- action (`auto_match|confirm|remap|split|ignore`)
- actor
- payload (jsonb)
- created_at

---

## Что увидишь в первой демке
1. Dashboard с живыми KPI-заглушками и drilldown
2. Identity Mapping с реальными конфликтами + confirm/remap
3. Person 360 timeline
4. Ask Ovia с цитатами
5. Weekly report preview

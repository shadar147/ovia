import type { Messages } from "../types";

export const messages: Messages = {
  // ── Навигация ──
  "nav.dashboard": "Дашборд",
  "nav.identityMapping": "Маппинг идентичностей",
  "nav.askOvia": "Спросить Ovia",
  "nav.reports": "Отчёты",
  "nav.settings": "Настройки",

  // ── Верхняя панель ──
  "topbar.org": "орг: {org}",

  // ── Дашборд ──
  "dashboard.title": "Дашборд",
  "dashboard.loading": "Загрузка метрик...",
  "dashboard.weekOf": "Неделя {period}",
  "dashboard.failedToLoad": "Не удалось загрузить данные KPI",

  // ── Графики дашборда ──
  "dashboard.throughputTrend": "Тренд пропускной способности",
  "dashboard.throughputTrendDesc": "Выполненные задачи по неделям: фичи, баги, техдолг",
  "dashboard.latencyTrend": "Тренд задержки ревью",
  "dashboard.latencyTrendDesc": "Время от открытия MR до первого ревью. Медиана (сплошная) и P90 (пунктир)",
  "dashboard.healthOverTime": "Здоровье поставки во времени",
  "dashboard.healthOverTimeDesc": "Комплексная оценка (0-100) = 30% пропускная способность + 30% задержка ревью + 20% количество блокеров + 20% доля переноса. Выше 80 = Здоров, 60-80 = Под угрозой, ниже 60 = Критично",
  "dashboard.topRisks": "Главные риски и блокеры",
  "dashboard.topRisksDesc": "Активные блокеры, устаревшие PR и сбойные пайплайны. Оценка риска = 40% возраст блокера + 30% сбойные пайплайны + 30% устаревшие MR",
  "dashboard.throughputMix": "Структура пропускной способности",
  "dashboard.throughputMixDesc": "Распределение задач текущей недели по типам",

  // ── KPI карточки ──
  "kpi.deliveryHealth": "Здоровье поставки",
  "kpi.noData": "Нет данных",
  "kpi.releaseRisk": "Риск релиза",
  "kpi.lowerIsBetter": "Чем меньше, тем лучше",
  "kpi.throughput": "Пропускная способность",
  "kpi.itemsDelivered": "задач выполнено",
  "kpi.reviewLatency": "Задержка ревью",
  "kpi.vsPrevWeek": "к пред. неделе",

  // ── Описания KPI ──
  "kpi.deliveryHealthDesc": "Взвешенная оценка 0-100:\n30% Пропускная способность (макс. 100)\n30% Задержка ревью (0ч=100, 48ч+=0)\n20% Блокеры (0=100, 10+=0)\n20% Доля переноса (0%=100, 100%=0)",
  "kpi.releaseRiskDesc": "Взвешенная оценка 0-100:\n40% Возраст блокера (кол-во x10 + дней x0.5)\n30% Сбойные пайплайны (каждый = 20 баллов)\n30% Устаревшие MR\nМетки: <35 Низкий, 35-70 Средний, >70 Высокий",
  "kpi.throughputDesc": "Общее количество выполненных задач за период.\nРаспределение: фичи + баги + техдолг.\nИсточник: закрытые задачи Jira / слитые MR.",
  "kpi.latencyDesc": "Медианное время от открытия MR до первого ревью.\nP90 = 90-й перцентиль (худшие 10% ревью).\nЦель: медиана <4ч, P90 <12ч.",

  // ── Здоровье ──
  "health.healthy": "Здоров",
  "health.atRisk": "Под угрозой",
  "health.critical": "Критично",

  // ── Легенда графиков ──
  "chart.features": "Фичи",
  "chart.bugs": "Баги",
  "chart.chores": "Техдолг",
  "chart.median": "Медиана",
  "chart.p90": "P90",
  "chart.deliveryHealth": "Здоровье поставки",
  "chart.hours": "Часы",
  "chart.healthyThreshold": "Здоров (80)",
  "chart.atRiskThreshold": "Под угрозой (60)",

  // ── Таблица рисков ──
  "risk.type": "Тип",
  "risk.title": "Название",
  "risk.owner": "Владелец",
  "risk.age": "Возраст",
  "risk.status": "Статус",
  "risk.noRisks": "Рисков не обнаружено — отличная работа!",
  "risk.unassigned": "Не назначен",

  // ── Страница идентичностей ──
  "identity.title": "Маппинг идентичностей",
  "identity.subtitle": "Сопоставление учётных записей Jira, GitLab и Confluence с каноническими людьми.",
  "identity.bulkConfirm": "Подтвердить все ({count})",
  "identity.exportCsv": "Экспорт CSV",
  "identity.noMappings": "Нет маппингов",
  "identity.noMappingsDesc": "Запустите синхронизацию для создания маппингов идентичностей.",
  "identity.noAutoLinks": "Нет авто-связей для подтверждения",
  "identity.remapSoon": "Переназначение скоро будет доступно",
  "identity.exportFailed": "Не удалось экспортировать CSV",
  "identity.failedToLoad": "Не удалось загрузить маппинги",

  // ── Фильтры идентичностей ──
  "identity.filterStatus": "Статус",
  "identity.filterAll": "Все",
  "identity.filterConfidence": "Уверенность: {min}% – {max}%",
  "identity.filterReset": "Сбросить",

  // ── Таблица идентичностей ──
  "identity.colPerson": "Человек",
  "identity.colIdentity": "Идентичность",
  "identity.colStatus": "Статус",
  "identity.colConfidence": "Уверенность",
  "identity.colUpdated": "Обновлено",
  "identity.noMappingsFound": "Маппинги не найдены.",
  "identity.totalCount": "{count} всего",
  "identity.previous": "Назад",
  "identity.next": "Далее",

  // ── Панель деталей ──
  "identity.drawerTitle": "Детали маппинга",
  "identity.person": "Человек",
  "identity.identity": "Идентичность",
  "identity.matchRationale": "Обоснование совпадения",
  "identity.totalConfidence": "Общая уверенность",
  "identity.audit": "Аудит",
  "identity.created": "Создано: {date}",
  "identity.updated": "Обновлено: {date}",
  "identity.confirm": "Подтвердить",
  "identity.remap": "Переназначить",
  "identity.split": "Разделить",
  "identity.unknown": "Неизвестно",
  "identity.serviceAccount": "Сервисная учётная запись",

  // ── Статусные бейджи ──
  "status.auto": "Авто",
  "status.verified": "Проверен",
  "status.conflict": "Конфликт",
  "status.rejected": "Отклонён",
  "status.split": "Разделён",

  // ── Компоненты состояния ──
  "state.error": "Ошибка",
  "state.somethingWrong": "Что-то пошло не так.",
  "state.retry": "Повторить",
  "state.noData": "Нет данных",
  "state.nothingToShow": "Здесь пока ничего нет.",

  // ── Заглушки страниц ──
  "ask.title": "Спросить Ovia",
  "ask.description": "Вопросы с цитированием — появятся в Фазе 4.",
  "reports.title": "Отчёты",
  "reports.description": "Шаблоны отчётов и расписание — появятся в Фазе 5.",
  "settings.title": "Настройки",
  "settings.description": "Подключения, политика синхронизации, роли — появятся в Фазе 5.",
};

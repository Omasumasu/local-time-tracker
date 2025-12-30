use chrono::NaiveDate;
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;
use crate::AppState;

/// タスク別の集計データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub task_id: Option<Uuid>,
    pub task_name: String,
    pub task_color: String,
    pub total_seconds: i64,
    pub entry_count: i64,
}

/// 日別の集計データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub total_seconds: i64,
    pub entry_count: i64,
}

/// 月次レポートデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyReport {
    pub year: i32,
    pub month: u32,
    pub total_seconds: i64,
    pub total_entries: i64,
    pub working_days: i64,
    pub average_seconds_per_day: i64,
    pub task_summaries: Vec<TaskSummary>,
    pub daily_summaries: Vec<DailySummary>,
}

/// 月次レポートを取得する
fn fetch_monthly_report(conn: &Connection, year: i32, month: u32) -> AppResult<MonthlyReport> {
    // 月の開始日と終了日を計算
    let start_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let end_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    };

    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();

    // タスク別集計
    let task_summaries = fetch_task_summaries(conn, &start_str, &end_str)?;

    // 日別集計
    let daily_summaries = fetch_daily_summaries(conn, &start_str, &end_str)?;

    // 全体集計
    let total_seconds: i64 = task_summaries.iter().map(|t| t.total_seconds).sum();
    let total_entries: i64 = task_summaries.iter().map(|t| t.entry_count).sum();
    let working_days = daily_summaries.len() as i64;
    let average_seconds_per_day = if working_days > 0 {
        total_seconds / working_days
    } else {
        0
    };

    Ok(MonthlyReport {
        year,
        month,
        total_seconds,
        total_entries,
        working_days,
        average_seconds_per_day,
        task_summaries,
        daily_summaries,
    })
}

/// タスク別の集計を取得
fn fetch_task_summaries(conn: &Connection, start: &str, end: &str) -> AppResult<Vec<TaskSummary>> {
    let sql = r#"
        SELECT
            e.task_id,
            COALESCE(t.name, '未分類') as task_name,
            COALESCE(t.color, '#6b7280') as task_color,
            SUM(
                CASE
                    WHEN e.ended_at IS NOT NULL
                    THEN EPOCH(e.ended_at::TIMESTAMP) - EPOCH(e.started_at::TIMESTAMP)
                    ELSE 0
                END
            )::BIGINT as total_seconds,
            COUNT(*)::BIGINT as entry_count
        FROM time_entries e
        LEFT JOIN tasks t ON e.task_id = t.id
        WHERE CAST(e.started_at::TIMESTAMP AS DATE) >= ? AND CAST(e.started_at::TIMESTAMP AS DATE) < ?
          AND e.ended_at IS NOT NULL
        GROUP BY e.task_id, t.name, t.color
        ORDER BY total_seconds DESC
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, end], |row| {
        let task_id_str: Option<String> = row.get(0)?;
        Ok(TaskSummary {
            task_id: task_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
            task_name: row.get(1)?,
            task_color: row.get(2)?,
            total_seconds: row.get(3)?,
            entry_count: row.get(4)?,
        })
    })?;

    let mut summaries = Vec::new();
    for row in rows {
        summaries.push(row?);
    }
    Ok(summaries)
}

/// 日別の集計を取得
fn fetch_daily_summaries(conn: &Connection, start: &str, end: &str) -> AppResult<Vec<DailySummary>> {
    let sql = r#"
        SELECT
            CAST(CAST(started_at::TIMESTAMP AS DATE) AS VARCHAR) as date,
            SUM(
                CASE
                    WHEN ended_at IS NOT NULL
                    THEN EPOCH(ended_at::TIMESTAMP) - EPOCH(started_at::TIMESTAMP)
                    ELSE 0
                END
            )::BIGINT as total_seconds,
            COUNT(*)::BIGINT as entry_count
        FROM time_entries
        WHERE CAST(started_at::TIMESTAMP AS DATE) >= ? AND CAST(started_at::TIMESTAMP AS DATE) < ?
          AND ended_at IS NOT NULL
        GROUP BY CAST(started_at::TIMESTAMP AS DATE)
        ORDER BY date ASC
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([start, end], |row| {
        Ok(DailySummary {
            date: row.get(0)?,
            total_seconds: row.get(1)?,
            entry_count: row.get(2)?,
        })
    })?;

    let mut summaries = Vec::new();
    for row in rows {
        summaries.push(row?);
    }
    Ok(summaries)
}

/// 利用可能な月のリストを取得
fn fetch_available_months(conn: &Connection) -> AppResult<Vec<(i32, u32)>> {
    let sql = r#"
        SELECT DISTINCT
            EXTRACT(YEAR FROM started_at)::INTEGER as year,
            EXTRACT(MONTH FROM started_at)::INTEGER as month
        FROM time_entries
        WHERE ended_at IS NOT NULL
        ORDER BY year DESC, month DESC
    "#;

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)? as u32))
    })?;

    let mut months = Vec::new();
    for row in rows {
        months.push(row?);
    }
    Ok(months)
}

/// 月次レポートを取得する
#[tauri::command]
pub fn get_monthly_report(
    state: tauri::State<AppState>,
    year: i32,
    month: u32,
) -> AppResult<MonthlyReport> {
    state
        .db
        .with_connection(|conn| fetch_monthly_report(conn, year, month))
}

/// 利用可能な月のリストを取得する
#[tauri::command]
pub fn get_available_months(state: tauri::State<AppState>) -> AppResult<Vec<(i32, u32)>> {
    state.db.with_connection(fetch_available_months)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn create_test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    mod monthly_report_tests {
        use super::*;

        #[test]
        fn 空のデータベースから月次レポートを取得すると空のレポートが返る() {
            let db = create_test_db();

            let report = db
                .with_connection(|conn| fetch_monthly_report(conn, 2024, 12))
                .unwrap();

            assert_eq!(report.year, 2024);
            assert_eq!(report.month, 12);
            assert_eq!(report.total_seconds, 0);
            assert_eq!(report.total_entries, 0);
            assert!(report.task_summaries.is_empty());
            assert!(report.daily_summaries.is_empty());
        }

        #[test]
        fn 時間記録がある月のレポートを取得できる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                // タスクを作成
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at)
                     VALUES ('task-1', 'テストタスク', '#ff0000', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;

                // 時間記録を作成（1時間）
                conn.execute(
                    "INSERT INTO time_entries (id, task_id, started_at, ended_at, created_at, updated_at)
                     VALUES ('entry-1', 'task-1',
                             '2024-12-15 09:00:00+00',
                             '2024-12-15 10:00:00+00',
                             CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;

                Ok(())
            })
            .unwrap();

            let report = db
                .with_connection(|conn| fetch_monthly_report(conn, 2024, 12))
                .unwrap();

            assert_eq!(report.total_seconds, 3600);
            assert_eq!(report.total_entries, 1);
            assert_eq!(report.working_days, 1);
            assert_eq!(report.task_summaries.len(), 1);
            assert_eq!(report.task_summaries[0].task_name, "テストタスク");
        }

        #[test]
        fn タスク別に集計される() {
            let db = create_test_db();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO tasks (id, name, color, created_at, updated_at) VALUES
                     ('task-1', 'タスクA', '#ff0000', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                     ('task-2', 'タスクB', '#00ff00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;

                conn.execute(
                    "INSERT INTO time_entries (id, task_id, started_at, ended_at, created_at, updated_at) VALUES
                     ('entry-1', 'task-1', '2024-12-15 09:00:00+00', '2024-12-15 10:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                     ('entry-2', 'task-1', '2024-12-15 11:00:00+00', '2024-12-15 12:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                     ('entry-3', 'task-2', '2024-12-15 14:00:00+00', '2024-12-15 15:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;

                Ok(())
            })
            .unwrap();

            let report = db
                .with_connection(|conn| fetch_monthly_report(conn, 2024, 12))
                .unwrap();

            assert_eq!(report.task_summaries.len(), 2);
            // タスクAが2時間で最初
            assert_eq!(report.task_summaries[0].task_name, "タスクA");
            assert_eq!(report.task_summaries[0].total_seconds, 7200);
            assert_eq!(report.task_summaries[0].entry_count, 2);
            // タスクBが1時間
            assert_eq!(report.task_summaries[1].task_name, "タスクB");
            assert_eq!(report.task_summaries[1].total_seconds, 3600);
        }

        #[test]
        fn 日別に集計される() {
            let db = create_test_db();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO time_entries (id, started_at, ended_at, created_at, updated_at) VALUES
                     ('entry-1', '2024-12-15 09:00:00+00', '2024-12-15 10:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                     ('entry-2', '2024-12-16 09:00:00+00', '2024-12-16 11:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

            let report = db
                .with_connection(|conn| fetch_monthly_report(conn, 2024, 12))
                .unwrap();

            assert_eq!(report.daily_summaries.len(), 2);
            assert_eq!(report.daily_summaries[0].date, "2024-12-15");
            assert_eq!(report.daily_summaries[0].total_seconds, 3600);
            assert_eq!(report.daily_summaries[1].date, "2024-12-16");
            assert_eq!(report.daily_summaries[1].total_seconds, 7200);
        }

        #[test]
        fn 未分類のエントリも集計される() {
            let db = create_test_db();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO time_entries (id, started_at, ended_at, created_at, updated_at)
                     VALUES ('entry-1', '2024-12-15 09:00:00+00', '2024-12-15 10:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

            let report = db
                .with_connection(|conn| fetch_monthly_report(conn, 2024, 12))
                .unwrap();

            assert_eq!(report.task_summaries.len(), 1);
            assert_eq!(report.task_summaries[0].task_name, "未分類");
        }
    }

    mod available_months_tests {
        use super::*;

        #[test]
        fn 利用可能な月のリストを取得できる() {
            let db = create_test_db();

            db.with_connection(|conn| {
                conn.execute(
                    "INSERT INTO time_entries (id, started_at, ended_at, created_at, updated_at) VALUES
                     ('entry-1', '2024-11-15 09:00:00+00', '2024-11-15 10:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                     ('entry-2', '2024-12-15 09:00:00+00', '2024-12-15 10:00:00+00', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                    [],
                )?;
                Ok(())
            })
            .unwrap();

            let months = db.with_connection(fetch_available_months).unwrap();

            assert_eq!(months.len(), 2);
            assert_eq!(months[0], (2024, 12));
            assert_eq!(months[1], (2024, 11));
        }
    }
}

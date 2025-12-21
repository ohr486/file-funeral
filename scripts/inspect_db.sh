#!/bin/bash
# SQLite データベース確認スクリプト

DB_PATH="${1:-$HOME/.file-funeral/sync_history.db}"

if [ ! -f "$DB_PATH" ]; then
    echo "❌ Database not found: $DB_PATH"
    echo ""
    echo "Usage: $0 [database_path]"
    echo "Default: $HOME/.file-funeral/sync_history.db"
    exit 1
fi

echo "📊 File Funeral - Sync History Database Inspector"
echo "=================================================="
echo "Database: $DB_PATH"
echo "Size: $(ls -lh "$DB_PATH" | awk '{print $5}')"
echo ""

# Schema version
echo "🔢 Schema Version:"
sqlite3 "$DB_PATH" "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1;"
echo ""

# Total sync count
echo "📈 Total Sync Operations:"
sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sync_history;"
echo ""

# Success/Failure stats
echo "✅ Success/Failure Statistics:"
sqlite3 -header -column "$DB_PATH" \
  "SELECT
    CASE WHEN success = 1 THEN '✅ Success' ELSE '❌ Failed' END as status,
    COUNT(*) as count,
    SUM(files_uploaded) as uploaded,
    SUM(files_downloaded) as downloaded,
    SUM(files_deleted) as deleted
   FROM sync_history
   GROUP BY success;"
echo ""

# Recent syncs
echo "🕐 Recent Sync History (Last 10):"
sqlite3 -header -column "$DB_PATH" \
  "SELECT
    id,
    datetime(sync_completed_at) as completed,
    files_uploaded as up,
    files_downloaded as down,
    files_deleted as del,
    conflicts_resolved as conf,
    CASE WHEN success = 1 THEN '✅' ELSE '❌' END as ok
   FROM sync_history
   ORDER BY sync_completed_at DESC
   LIMIT 10;"
echo ""

# Total files tracked
echo "📁 Total Files Tracked:"
sqlite3 "$DB_PATH" \
  "SELECT COUNT(DISTINCT file_path) FROM synced_files;"
echo ""

echo "💡 Tips:"
echo "  - Open interactive shell: sqlite3 $DB_PATH"
echo "  - View all tables: sqlite3 $DB_PATH '.tables'"
echo "  - Export to CSV: sqlite3 -header -csv $DB_PATH 'SELECT * FROM sync_history;'"

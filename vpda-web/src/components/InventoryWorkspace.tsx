import { useState, useEffect } from 'react';
import type { VpdaAppState } from '../data/useVpdaApp';

type DatabaseInfo = {
  name: string;
  path: string;
  size: number;
  type: string;
};

type QueryResult = {
  columns: string[];
  rows: any[][];
  rowCount: number;
  executionTime: number;
};

type Props = {
  app: VpdaAppState;
};

export default function InventoryWorkspace({ app }: Props) {
  const [databases, setDatabases] = useState<DatabaseInfo[]>([]);
  const [selectedDb, setSelectedDb] = useState<string>('');
  const [query, setQuery] = useState<string>('SELECT * FROM sqlite_master WHERE type="table";');
  const [results, setResults] = useState<QueryResult | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>('');
  const [loadingDbs, setLoadingDbs] = useState<boolean>(true);
  const [isShowingTables, setIsShowingTables] = useState<boolean>(false);

  // Fetch available databases on mount
  useEffect(() => {
    fetchDatabases();
  }, []);

  async function fetchDatabases() {
    setLoadingDbs(true);
    setError('');
    try {
      const response = await fetch('http://127.0.0.1:3017/api/inventory/databases');
      if (!response.ok) {
        throw new Error(`Failed to fetch databases: ${response.statusText}`);
      }
      const data = await response.json();
      setDatabases(data.databases || []);
      if (data.databases && data.databases.length > 0) {
        setSelectedDb(data.databases[0].path);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch databases');
    } finally {
      setLoadingDbs(false);
    }
  }

  async function executeQuery() {
    if (!selectedDb) {
      setError('Please select a database first');
      return;
    }
    if (!query.trim()) {
      setError('Please enter a query');
      return;
    }

    setLoading(true);
    setError('');
    setResults(null);
    setIsShowingTables(false);

    try {
      const response = await fetch('http://127.0.0.1:3017/api/inventory/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          database: selectedDb,
          query: query.trim(),
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Query failed: ${response.statusText}`);
      }

      const data = await response.json();
      setResults(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Query execution failed');
    } finally {
      setLoading(false);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLTextAreaElement>) {
    // Ctrl/Cmd + Enter to execute
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      executeQuery();
    }
  }

  async function showTablesQuery() {
    if (!selectedDb) {
      setError('Please select a database first');
      return;
    }

    const tablesQuery = 'SELECT name FROM sqlite_master WHERE type="table" ORDER BY name;';
    setQuery(tablesQuery);
    setIsShowingTables(true);

    setLoading(true);
    setError('');
    setResults(null);

    try {
      const response = await fetch('http://127.0.0.1:3017/api/inventory/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          database: selectedDb,
          query: tablesQuery,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Query failed: ${response.statusText}`);
      }

      const data = await response.json();
      setResults(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Query execution failed');
    } finally {
      setLoading(false);
    }
  }

  async function selectFromTable(tableName: string) {
    if (!selectedDb) return;

    const selectQuery = `SELECT * FROM "${tableName}" LIMIT 100;`;
    setQuery(selectQuery);
    setIsShowingTables(false);

    setLoading(true);
    setError('');
    setResults(null);

    try {
      const response = await fetch('http://127.0.0.1:3017/api/inventory/query', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          database: selectedDb,
          query: selectQuery,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || `Query failed: ${response.statusText}`);
      }

      const data = await response.json();
      setResults(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Query execution failed');
    } finally {
      setLoading(false);
    }
  }

  return (
    <div style={{
      width: '100%',
      height: '100%',
      display: 'flex',
      flexDirection: 'column',
      background: 'var(--bg0)',
      color: 'var(--text)',
      overflow: 'hidden',
    }}>
      {/* Header */}
      <div style={{
        padding: '16px 20px',
        borderBottom: '1px solid var(--border1)',
        background: 'var(--bg1)',
      }}>
        <h2 style={{
          margin: 0,
          fontSize: 'var(--fs-13)',
          fontWeight: 600,
          color: 'var(--text)',
        }}>
          Database Inventory
        </h2>
        <p style={{
          margin: '4px 0 0 0',
          fontSize: 'var(--fs-10)',
          color: 'var(--text-muted)',
          opacity: 0.7,
        }}>
          Query and explore VPDA databases (.shot, .shot_state, and other SQLite files)
        </p>
      </div>

      {/* Content */}
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        padding: '20px',
        gap: '16px',
        overflowY: 'auto',
      }}>
        {/* Database Selector */}
        <div>
          <label style={{
            display: 'block',
            marginBottom: '8px',
            fontSize: 'var(--fs-11)',
            fontWeight: 500,
            color: 'var(--text)',
          }}>
            Select Database
          </label>
          {loadingDbs ? (
            <div style={{ padding: '8px', color: 'var(--text-muted)' }}>
              Loading databases...
            </div>
          ) : databases.length === 0 ? (
            <div style={{ padding: '8px', color: 'var(--text-muted)' }}>
              No databases found. Check if database files exist in the project.
            </div>
          ) : (
            <select
              value={selectedDb}
              onChange={(e) => setSelectedDb(e.target.value)}
              style={{
                width: '100%',
                padding: '8px 12px',
                fontSize: 'var(--fs-11)',
                background: 'var(--bg1)',
                color: 'var(--text)',
                border: '1px solid var(--border2)',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              {databases.map((db) => (
                <option key={db.path} value={db.path}>
                  {db.name} ({db.type}) - {(db.size / 1024).toFixed(1)} KB
                </option>
              ))}
            </select>
          )}
          <button
            className="ue-btn small"
            onClick={fetchDatabases}
          >
            Refresh Database List
          </button>
          <button
            className="ue-btn small"
            onClick={showTablesQuery}
            disabled={!selectedDb}
          >
            Show Tables
          </button>
        </div>

        {/* Query Input */}
        <div style={{ flex: '0 0 auto' }}>
          <label style={{
            display: 'block',
            marginBottom: '8px',
            fontSize: 'var(--fs-11)',
            fontWeight: 500,
            color: 'var(--text)',
          }}>
            SQL Query <span style={{ opacity: 0.6, fontWeight: 400 }}>(Ctrl/Cmd + Enter to execute)</span>
          </label>
          <textarea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Enter SQL query..."
            spellCheck={false}
            style={{
              width: '100%',
              height: '120px',
              padding: '12px',
              fontSize: 'var(--fs-11)',
              fontFamily: 'ui-monospace, monospace',
              background: 'var(--bg1)',
              color: 'var(--text)',
              border: '1px solid var(--border2)',
              borderRadius: '4px',
              resize: 'vertical',
              lineHeight: '1.5',
            }}
          />
          <button
            className="ue-btn small"
            onClick={executeQuery}
            disabled={loading || !selectedDb}
          >
            {loading ? 'Executing...' : 'Execute Query'}
          </button>
        </div>

        {/* Error Display */}
        {error && (
          <div style={{
            padding: '12px',
            background: 'rgba(239, 68, 68, 0.1)',
            border: '1px solid rgba(239, 68, 68, 0.3)',
            borderRadius: '4px',
            color: '#ef4444',
            fontSize: 'var(--fs-11)',
          }}>
            <strong>Error:</strong> {error}
          </div>
        )}

        {/* Results Display */}
        {results && (
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
            <div style={{
              marginBottom: '12px',
              fontSize: 'var(--fs-11)',
              color: 'var(--text-muted)',
            }}>
              {results.rowCount} row{results.rowCount !== 1 ? 's' : ''} returned in {results.executionTime.toFixed(2)}ms
            </div>
            <div style={{
              flex: 1,
              overflowX: 'auto',
              overflowY: 'auto',
              border: '1px solid var(--border2)',
              borderRadius: '4px',
              background: 'var(--bg1)',
            }}>
              <table style={{
                width: '100%',
                borderCollapse: 'collapse',
                fontSize: 'var(--fs-10)',
              }}>
                <thead>
                  <tr style={{
                    background: 'var(--bg2)',
                    position: 'sticky',
                    top: 0,
                    zIndex: 1,
                  }}>
                    {results.columns.map((col, idx) => (
                      <th key={idx} style={{
                        padding: '10px 12px',
                        textAlign: 'left',
                        fontWeight: 600,
                        color: 'var(--text)',
                        borderBottom: '1px solid var(--border2)',
                        whiteSpace: 'nowrap',
                      }}>
                        {col}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {results.rows.map((row, rowIdx) => (
                    <tr key={rowIdx} style={{
                      borderBottom: '1px solid var(--border1)',
                    }}>
                      {row.map((cell, cellIdx) => (
                        <td
                          key={cellIdx}
                          onClick={isShowingTables && cellIdx === 0 ? () => selectFromTable(String(cell)) : undefined}
                          style={{
                            padding: '8px 12px',
                            color: isShowingTables && cellIdx === 0 ? 'var(--accent)' : 'var(--text)',
                            maxWidth: '300px',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                            cursor: isShowingTables && cellIdx === 0 ? 'pointer' : 'default',
                            textDecoration: isShowingTables && cellIdx === 0 ? 'underline' : 'none',
                          }}
                          title={String(cell)}
                        >
                          {cell === null ? (
                            <span style={{ opacity: 0.5, fontStyle: 'italic' }}>NULL</span>
                          ) : typeof cell === 'boolean' ? (
                            <span style={{ color: cell ? '#22c55e' : '#ef4444' }}>
                              {cell ? 'true' : 'false'}
                            </span>
                          ) : (
                            String(cell)
                          )}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

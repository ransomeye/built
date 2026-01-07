# Path and File Name : /home/ransomeye/rebuild/ui/schema_helper.py
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Schema-aware helper functions for fail-soft database queries

"""
Schema-aware database helper for RansomEye UI.
Provides fail-soft query execution with column existence checks.
"""

import psycopg2
import logging
from typing import Optional, Dict, List, Any, Tuple

logger = logging.getLogger(__name__)


class SchemaAwareDB:
    """Schema-aware database helper with fail-soft query execution."""
    
    def __init__(self, conn: psycopg2.extensions.connection):
        self.conn = conn
        self._column_cache: Dict[str, set] = {}
    
    def column_exists(self, table_schema: str, table_name: str, column_name: str) -> bool:
        """Check if a column exists in a table using information_schema."""
        cache_key = f"{table_schema}.{table_name}"
        
        if cache_key not in self._column_cache:
            try:
                cursor = self.conn.cursor()
                cursor.execute("""
                    SELECT column_name
                    FROM information_schema.columns
                    WHERE table_schema = %s AND table_name = %s
                """, (table_schema, table_name))
                self._column_cache[cache_key] = {row[0] for row in cursor.fetchall()}
                cursor.close()
            except Exception as e:
                logger.error(f"Error checking columns for {cache_key}: {e}", exc_info=True)
                return False
        
        return column_name in self._column_cache.get(cache_key, set())
    
    def table_exists(self, table_schema: str, table_name: str) -> bool:
        """Check if a table exists."""
        try:
            cursor = self.conn.cursor()
            cursor.execute("""
                SELECT EXISTS (
                    SELECT 1 FROM information_schema.tables
                    WHERE table_schema = %s AND table_name = %s
                )
            """, (table_schema, table_name))
            result = cursor.fetchone()[0]
            cursor.close()
            return result
        except Exception as e:
            logger.error(f"Error checking table existence {table_schema}.{table_name}: {e}", exc_info=True)
            try:
                self.conn.rollback()
            except:
                pass
            return False
    
    def safe_query(self, query: str, params: Optional[Tuple] = None, 
                   default: Any = None) -> Optional[Any]:
        """Execute a query with error handling - returns default on error."""
        try:
            cursor = self.conn.cursor()
            cursor.execute(query, params or ())
            result = cursor.fetchone()
            cursor.close()
            return result[0] if result else default
        except Exception as e:
            logger.error(f"Query failed: {query[:100]}... Error: {e}", exc_info=True)
            try:
                self.conn.rollback()
            except:
                pass
            return default
    
    def safe_query_all(self, query: str, params: Optional[Tuple] = None, 
                      default: List = None) -> List:
        """Execute a query returning multiple rows - returns default on error."""
        try:
            cursor = self.conn.cursor()
            cursor.execute(query, params or ())
            result = cursor.fetchall()
            cursor.close()
            return result
        except Exception as e:
            logger.error(f"Query failed: {query[:100]}... Error: {e}", exc_info=True)
            try:
                self.conn.rollback()
            except:
                pass
            return default or []
    
    def safe_count(self, table_schema: str, table_name: str, 
                   where_clause: str = "", params: Optional[Tuple] = None) -> int:
        """Safely count rows in a table."""
        if not self.table_exists(table_schema, table_name):
            return 0
        
        query = f"SELECT COUNT(*) FROM {table_schema}.{table_name}"
        if where_clause:
            query += f" WHERE {where_clause}"
        
        return self.safe_query(query, params, default=0) or 0
    
    def get_column_value(self, table_schema: str, table_name: str, 
                        column_name: str, where_clause: str = "",
                        params: Optional[Tuple] = None, default: Any = None) -> Any:
        """Safely get a column value if it exists."""
        if not self.column_exists(table_schema, table_name, column_name):
            return default
        
        query = f"SELECT {column_name} FROM {table_schema}.{table_name}"
        if where_clause:
            query += f" WHERE {where_clause}"
        query += " LIMIT 1"
        
        return self.safe_query(query, params, default=default)


def safe_execute_query(conn: psycopg2.extensions.connection, query: str, 
                      params: Optional[Tuple] = None, default: Any = None) -> Any:
    """Convenience function for safe query execution."""
    db = SchemaAwareDB(conn)
    return db.safe_query(query, params, default)


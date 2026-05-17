import sqlite3
conn = sqlite3.connect('.contextflow/memory.db')
cursor = conn.cursor()
cursor.execute("SELECT * FROM events")
rows = cursor.fetchall()
print(f"Total Events: {len(rows)}")
for row in rows:
    print(row)
conn.close()

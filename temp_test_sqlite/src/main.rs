use rusqlite::Connection;

#[test]
fn test_sqlite() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", []).unwrap();
}

fn main() {}

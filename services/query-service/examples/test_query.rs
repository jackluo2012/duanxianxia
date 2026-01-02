use clickhouse::Client;
use clickhouse::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default()
        .with_url("http://localhost:8123")
        .with_user("default")
        .with_password("")
        .with_database("duanxianxia");

    // 方法1: 使用 query().fetch()
    println!("方法1: query().fetch()");
    let mut cursor = client.query("SELECT 1 as num, 'test' as str").fetch()?;

    while let Some(row) = cursor.next().await? {
        let num: u8 = row.get("num")?;
        let str: String = row.get("str")?;
        println!("num={}, str={}", num, str);
    }

    // 方法2: 使用 query().fetch_all()
    println!("\n方法2: query().fetch_all()");
    let rows = client
        .query("SELECT 2 as num")
        .fetch_all::<Row<[u8]>>()
        .await?;

    for row in rows {
        println!("num={:?}", row);
    }

    Ok(())
}

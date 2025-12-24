use anyhow::{Result};
use futures::StreamExt;
use neo4rs::{Query, Txn};



pub async fn neo4j_count(tx: &mut Txn, q: Query) -> Result<i64> {
    let mut stream = tx.execute(q).await?;
    if let Some(row) = stream.next(tx).await? {
        let n: i64 = row.get("n")?;
        Ok(n)
    } else {
        Ok(0)
    }
}


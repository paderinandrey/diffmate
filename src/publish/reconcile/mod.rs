mod write;

use crate::error::Error;
use crate::gh::Gh;
use crate::pr_url::PrRef;
use crate::store::Store;

struct Row {
    id: String,
    state: String,
    owner: String,
    repo: String,
    number: u64,
}

pub(crate) async fn execute_reconcile(
    store: &Store,
    gh: &impl Gh,
    job_id: i64,
) -> Result<(), Error> {
    let row = load(store, job_id).await?;
    if row.state == "unknown" {
        match review_id(gh, &row).await {
            Ok(Some(review_id)) => return write::submit(store, job_id, &row.id, &review_id).await,
            Ok(None) => {}
            Err(error) => return write::miss(store, job_id, &error).await,
        }
    }
    write::succeed(store, job_id).await
}

async fn review_id(gh: &impl Gh, row: &Row) -> Result<Option<String>, Error> {
    let pr = PrRef {
        owner: row.owner.clone(),
        repo: row.repo.clone(),
        number: row.number,
    };
    let login = gh.current_login().await?;
    let marker = format!("<!-- diffmate:publication={} -->", row.id);
    let reviews = gh.list_my_reviews(&pr).await?;
    Ok(reviews
        .into_iter()
        .find(|review| review.author == login && review.body.contains(&marker))
        .map(|review| review.id))
}

async fn load(store: &Store, job_id: i64) -> Result<Row, Error> {
    store
        .call(move |conn| {
            let sql = "SELECT p.id, p.state, t.owner, t.repo, t.number FROM jobs j JOIN publications p ON p.id = j.publication_id JOIN targets t ON t.id = p.target_id WHERE j.id = ?1 AND j.kind = 'reconcile'";
            conn.query_row(sql, [job_id], |row| {
                let number: i64 = row.get(4)?;
                Ok(Row {
                    id: row.get(0)?,
                    state: row.get(1)?,
                    owner: row.get(2)?,
                    repo: row.get(3)?,
                    number: u64::try_from(number).map_err(number_err)?,
                })
            })
        })
        .await
}

fn number_err(error: std::num::TryFromIntError) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}

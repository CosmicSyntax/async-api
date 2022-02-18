use std::collections::VecDeque;

use tokio::sync::Mutex;
use tokio::task::yield_now;
use tokio_postgres::{Client, Error};

use crate::tls::tls_config;
pub mod models;

pub struct Psql {
    pub pool: Mutex<VecDeque<Client>>,
}

impl Psql {
    pub async fn new(pool_size: usize, url: String) -> Result<Self, Error> {
        let mut pool = VecDeque::with_capacity(pool_size);
        let config_tls = tls_config();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(config_tls);
        for _ in 0..pool_size {
            let (client, conn) = tokio_postgres::connect(&url, tls.clone()).await?;
            pool.push_back(client);
            // If the client is dropped, conn will go too
            tokio::spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("connection error: {}", e);
                }
            });
        }
        let guard = Psql {
            pool: Mutex::new(pool),
        };
        Ok(guard)
    }

    pub async fn set(&self, dbexec: &impl DbExec) {
        loop {
            
            let guard = self.pool.lock().await.pop_front();

            if let Some(client) = guard {
                let q = dbexec.set();
                let _ = client.execute(&q, &[]).await.unwrap();
                // put the client back into pool
                self.pool.lock().await.push_back(client);
                break;
            } else {
                yield_now().await;
                continue;
            }
        }
    }
}

pub trait DbExec {
    fn set(&self) -> String;
}

#[cfg(test)]
mod test {

    use self::super::*;
    use crate::Configuration;

    // Testing if we are able to make the connections to the data and pool them
    #[tokio::test]
    async fn test_run() {
        let pool_size = 2;
        let Configuration(conf) = Configuration::new("./configs/api.yml").await.unwrap();
        let conn = Psql::new(
            pool_size,
            conf[0]["db"]["url_host"].as_str().unwrap().into(),
        )
        .await
        .unwrap();
        // Check the pool size is correct
        assert_eq!(pool_size, conn.pool.lock().await.len());
    }
}

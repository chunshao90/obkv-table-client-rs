/*-
 * #%L
 * OBKV Table Client Framework
 * %%
 * Copyright (C) 2021 OceanBase
 * %%
 * OBKV Table Client Framework is licensed under Mulan PSL v2.
 * You can use this software according to the terms and conditions of the
 * Mulan PSL v2. You may obtain a copy of Mulan PSL v2 at:
 *          http://license.coscl.org.cn/MulanPSL2
 * THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
 * KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
 * NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 * See the Mulan PSL v2 for more details.
 * #L%
 */

extern crate obkv;

use std::{sync::Arc, time};

use obkv::{serde_obkv::value::Value, Builder, ObTableClient, RunningMode};
use tokio::task;

// TODO: use test conf to control which environments to test.
const TEST_FULL_USER_NAME: &str = "test";
const TEST_URL: &str = "127.0.0.1";
const TEST_PASSWORD: &str = "";
const TEST_SYS_USER_NAME: &str = "";
const TEST_SYS_PASSWORD: &str = "";

fn build_client(mode: RunningMode) -> ObTableClient {
    let builder = Builder::new()
        .full_user_name(TEST_FULL_USER_NAME)
        .param_url(TEST_URL)
        .running_mode(mode)
        .password(TEST_PASSWORD)
        .sys_user_name(TEST_SYS_USER_NAME)
        .sys_password(TEST_SYS_PASSWORD);

    let client = builder.build();

    assert!(client.is_ok());

    let client = client.unwrap();
    client.init().expect("Fail to create obkv client.");
    client
}

const TABLE_NAME: &str = "series_key_to_id_0";
// read and write the table:
// create table series_key_to_id_0 (
//  series_key VARBINARY(8096) NOT NULL,
//  series_id BIGINT NOT NULL,
//  PRIMARY KEY(series_key),
//  KEY index_id(series_id)
// );
async fn concurrent_insert(client: Arc<ObTableClient>) {
    let mut thds = Vec::with_capacity(20);
    for i in 0..50 {
        let client = client.clone();
        let thd = task::spawn(async move {
            for j in i * 100..(i * 100 + 50) {
                let series_key = format!("series_key_test_padding_padding_{j}");
                let series_id = j * j;
                client
                    .insert(
                        TABLE_NAME,
                        vec![Value::from(series_key.clone())],
                        vec!["series_id".to_owned()],
                        vec![Value::from(series_id as i64)],
                    )
                    .await
                    .unwrap_or_else(|err| {
                        panic!("fail to insert row:{series_key} {series_id}, err:{err}")
                    });
            }
        });
        thds.push(thd);
    }

    for (i, thd) in thds.into_iter().enumerate() {
        thd.await
            .unwrap_or_else(|_| panic!("thread#{i} fail to join"));
    }
}

#[tokio::main]
async fn main() {
    let client_handle = task::spawn_blocking(|| build_client(RunningMode::Normal));
    let client = client_handle.await.unwrap();
    client
        .truncate_table(TABLE_NAME)
        .expect("fail to truncate the table");
    let start = time::Instant::now();
    concurrent_insert(Arc::new(client)).await;
    let elapsed = time::Instant::now() - start;
    println!("Benches::concurrent_insert cost time:{elapsed:?}");
}

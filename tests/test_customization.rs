#![cfg(feature = "websocket")]
use std::{thread::sleep, time::Duration};

use async_dashscope::operation::audio::asr::{Customization, customization::VocabularyDetail};

pub mod common;

#[tokio::test]
async fn test_customization() -> anyhow::Result<()> {
    let client = common::init_client();
    let customization = Customization::new(client);
    // 添加热词
    let target_model = "fun-asr";
    let prefix = "testpfx";
    let voc = vec![("赛德克巴莱", 5), ("夏洛特烦恼", 5)];
    let voc = voc
        .into_iter()
        .map(VocabularyDetail::from)
        .collect::<Vec<_>>();
    let res = customization
        .create_vocabulary(target_model, prefix, &voc)
        .await?;

    let vocabulary_id = res.output.vocabulary_id;

    sleep(Duration::from_secs(2));

    // 查询热词
    // let res = customization
    //     .list_vocabularies(Some(prefix), None, None)
    //     .await?;
    // assert_eq!(res.output.vocabulary_list.len(), 1);

    // 查询热词详情
    let res = customization.query_vocabulary(&vocabulary_id).await?;
    assert_eq!(res.output.vocabulary.len(), 2);
    sleep(Duration::from_secs(2));
    // 更新热词
    let voc = vec![("赛德克巴莱", 5), ("夏洛特烦恼", 5), ("小西风", 5)];
    let voc = voc
        .into_iter()
        .map(VocabularyDetail::from)
        .collect::<Vec<_>>();
    let res = customization
        .update_vocabulary(&vocabulary_id, &voc)
        .await?;
    assert!(res.usage.count > 0);
    sleep(Duration::from_secs(2));
    // 删除热词
    let res = customization.delete_vocabulary(&vocabulary_id).await?;
    assert!(res.usage.count > 0);

    Ok(())
}

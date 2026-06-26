use anyhow::Result;
use ltp::{CWSModel, Codec, Format, ModelSerde, NERModel, POSModel};
use std::fs::File;
use std::path::Path;

const NEWS_SAMPLE: &str = r#"新华社北京6月26日电（记者李明）北京市人民政府今天召开新闻发布会，介绍城市交通治理和公共服务建设情况。北京市交通委员会主任张伟表示，今年下半年，北京将在朝阳区、海淀区和通州区新增多条公交线路，并继续推进地铁站周边慢行系统改造。

发布会上，国家发展和改革委员会有关负责人王磊介绍，相关部门将会同财政部、交通运输部，支持京津冀地区完善综合交通网络。天津市、河北省石家庄市和唐山市也将参与区域协同项目。

中国铁路北京局集团有限公司负责人赵敏表示，暑运期间，北京南站、北京西站和北京朝阳站将增加服务人员，并优化旅客进站流程。她说，中国铁路部门将根据客流变化动态调整列车运行方案。

此外，北京大学城市治理研究院教授陈晓在接受采访时表示，城市交通治理不仅需要基础设施建设，也需要企业、社区和公众共同参与。腾讯公司、美团公司和高德地图等企业已经参与部分试点项目。"#;

fn main() -> Result<()> {
    let model_dir = Path::new(".reading/models/ltp/legacy");
    let cws: CWSModel = ModelSerde::load(
        File::open(model_dir.join("cws_model.bin"))?,
        Format::AVRO(Codec::Deflate),
    )?;
    let pos: POSModel = ModelSerde::load(
        File::open(model_dir.join("pos_model.bin"))?,
        Format::AVRO(Codec::Deflate),
    )?;
    let ner: NERModel = ModelSerde::load(
        File::open(model_dir.join("ner_model.bin"))?,
        Format::AVRO(Codec::Deflate),
    )?;

    let words = cws.predict(NEWS_SAMPLE)?;
    let pos_tags = pos.predict(&words)?;
    let ner_tags = ner.predict((&words, &pos_tags))?;

    println!("=== ltp raw news sample: {} tokens ===", words.len());
    for ((word, pos_tag), ner_tag) in words.iter().zip(pos_tags.iter()).zip(ner_tags.iter()) {
        if *ner_tag != "O" || matches!(*pos_tag, "nh" | "ns" | "ni") {
            println!("{word}\t{pos_tag}\t{ner_tag}");
        }
    }

    Ok(())
}

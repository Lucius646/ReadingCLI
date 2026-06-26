use anyhow::Result;
use ltp::{CWSModel, Codec, Format, ModelSerde, NERModel, POSModel};
use reading_cli::highlight::analyzer::{AnalyzerKind, create_analyzer};
use std::fs::File;
use std::path::Path;

const SAMPLE: &str = r#"米切尔走进房间的时候，皮埃尔正坐在窗边看报纸。窗外的巴黎下着小雨，街角的咖啡馆亮着一盏黄色的灯。罗兰把一本小说放在桌上，说：“我们明天也许应该去马赛，或者去圣日耳曼大道附近看看。”米切尔没有回答，只是低头看着那封从里昂寄来的信。

信封上写着弗朗索瓦的名字，字迹很潦草。皮埃尔说，弗朗索瓦上个月还在布鲁塞尔，后来又去了日内瓦。他总是这样，从一个城市跑到另一个城市，好像每个地方都只是临时的站台。罗兰笑了笑，说：“你别把他说得像个逃犯，他只是讨厌安静。”

【米切尔】把信重新折好，夹进《南方高速》那本书里。桌上的旧报纸被风吹开，露出一条新闻：巴黎市政府将修复塞纳河边的一座旧桥。皮埃尔念了一遍标题，又把报纸推到一边，说：“这和我们没有关系。”

楼下传来杯子碰撞的声音。咖啡馆的服务员正在擦桌子，一个穿黑色外套的男人站在门口，似乎在等人。米切尔看了他一眼，觉得那个人有点像罗兰的朋友安德烈，可是又不敢确定。雨水沿着玻璃慢慢流下来，把街上的灯光拖成长长的影子。

“如果弗朗索瓦真的回到巴黎，”罗兰说，“他一定会先去蒙帕纳斯，而不是来这里找我们。”皮埃尔摇摇头，说：“你太了解他了，反而容易猜错。”米切尔终于开口：“不管他在哪儿，我们今晚都得把这封信读完。”

这时，电话响了。服务员抬起头，看向楼上的房间。米切尔没有动，皮埃尔也没有动。只有罗兰站起来，走到门边，像是早就知道电话会在这个时候响起。"#;

fn main() -> Result<()> {
    print_analyzer_annotations(AnalyzerKind::Jieba)?;
    print_analyzer_annotations(AnalyzerKind::LtpLegacyPos)?;
    print_analyzer_annotations(AnalyzerKind::LtpLegacyNer)?;
    print_ltp_raw()?;

    Ok(())
}

fn print_analyzer_annotations(kind: AnalyzerKind) -> Result<()> {
    let analyzer = create_analyzer(kind)?;
    let annotations = analyzer.analyze(SAMPLE, 0)?;

    println!();
    println!(
        "=== {} annotations: {} ===",
        analyzer.analyzer_id(),
        annotations.len()
    );

    for annotation in annotations.iter().take(120) {
        let text = &SAMPLE[annotation.start_offset as usize..annotation.end_offset as usize];
        println!(
            "{:?}\t{:>4}..{:<4}\t{}",
            annotation.kind, annotation.start_offset, annotation.end_offset, text
        );
    }

    Ok(())
}

fn print_ltp_raw() -> Result<()> {
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

    let words = cws.predict(SAMPLE)?;
    let pos_tags = pos.predict(&words)?;
    let ner_tags = ner.predict((&words, &pos_tags))?;

    println!();
    println!("=== ltp raw word / pos / ner: {} tokens ===", words.len());

    for ((word, pos_tag), ner_tag) in words.iter().zip(pos_tags.iter()).zip(ner_tags.iter()) {
        println!("{word}\t{pos_tag}\t{ner_tag}");
    }

    Ok(())
}

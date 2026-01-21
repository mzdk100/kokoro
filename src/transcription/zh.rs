/// 汉语拼音到国际音标的转换
/// 参考了python的misaki库的zh.py。
use std::{collections::HashMap, error::Error, fmt, sync::LazyLock};

const VALID_FINALS: [&str; 37] = [
    "i", "u", "ü", "a", "ia", "ua", "o", "uo", "e", "ie", "üe", "ai", "uai", "ei", "uei", "ao",
    "iao", "ou", "iou", "an", "ian", "uan", "üan", "en", "in", "uen", "ün", "ang", "iang", "uang",
    "eng", "ing", "ueng", "ong", "iong", "er", "ê",
];
const INITIALS: [&str; 21] = [
    "zh", "ch", "sh", "b", "c", "d", "f", "g", "h", "j", "k", "l", "m", "n", "p", "q", "r", "s",
    "t", "x", "z",
];

// 错误类型定义
#[derive(Debug)]
pub enum PinyinError {
    FinalNotFound(String),
}

impl fmt::Display for PinyinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PinyinError::FinalNotFound(tip) => write!(f, "Final not found: {}", tip),
        }
    }
}

impl Error for PinyinError {}

static INITIAL_MAPPING: LazyLock<HashMap<&'static str, Vec<Vec<&'static str>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        map.insert("b", vec![vec!["p"]]);
        map.insert("c", vec![vec!["ʦʰ"]]);
        map.insert("ch", vec![vec!["ꭧʰ"]]);
        map.insert("d", vec![vec!["t"]]);
        map.insert("f", vec![vec!["f"]]);
        map.insert("g", vec![vec!["k"]]);
        map.insert("h", vec![vec!["x"], vec!["h"]]);
        map.insert("j", vec![vec!["ʨ"]]);
        map.insert("k", vec![vec!["kʰ"]]);
        map.insert("l", vec![vec!["l"]]);
        map.insert("m", vec![vec!["m"]]);
        map.insert("n", vec![vec!["n"]]);
        map.insert("p", vec![vec!["pʰ"]]);
        map.insert("q", vec![vec!["ʨʰ"]]);
        map.insert("r", vec![vec!["ɻ"], vec!["ʐ"]]);
        map.insert("s", vec![vec!["s"]]);
        map.insert("sh", vec![vec!["ʂ"]]);
        map.insert("t", vec![vec!["tʰ"]]);
        map.insert("x", vec![vec!["ɕ"]]);
        map.insert("z", vec![vec!["ʦ"]]);
        map.insert("zh", vec![vec!["ꭧ"]]);
        map
    });

static SYLLABIC_CONSONANT_MAPPINGS: LazyLock<HashMap<&'static str, Vec<Vec<&'static str>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("hm", vec![vec!["h", "m0"]]);
        map.insert("hng", vec![vec!["h", "ŋ0"]]);
        map.insert("m", vec![vec!["m0"]]);
        map.insert("n", vec![vec!["n0"]]);
        map.insert("ng", vec![vec!["ŋ0"]]);
        map
    });

static INTERJECTION_MAPPINGS: LazyLock<HashMap<&'static str, Vec<Vec<&'static str>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("io", vec![vec!["j", "ɔ0"]]);
        map.insert("ê", vec![vec!["ɛ0"]]);
        map.insert("er", vec![vec!["ɚ0"], vec!["aɚ̯0"]]);
        map.insert("o", vec![vec!["ɔ0"]]);
        map
    });

/// Duanmu (2000, p. 37) and Lin (2007, p. 68f)
/// Diphtongs from Duanmu (2007, p. 40): au, əu, əi, ai
/// Diphthongs from Lin (2007, p. 68f): au̯, ou̯, ei̯, ai̯
static FINAL_MAPPING: LazyLock<HashMap<&'static str, Vec<Vec<&'static str>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("a", vec![vec!["a0"]]);
        map.insert("ai", vec![vec!["ai0"]]);
        map.insert("an", vec![vec!["a0", "n"]]);
        map.insert("ang", vec![vec!["a0", "ŋ"]]);
        map.insert("ao", vec![vec!["au0"]]);
        map.insert("e", vec![vec!["ɤ0"]]);
        map.insert("ei", vec![vec!["ei0"]]);
        map.insert("en", vec![vec!["ə0", "n"]]);
        map.insert("eng", vec![vec!["ə0", "ŋ"]]);
        map.insert("i", vec![vec!["i0"]]);
        map.insert("ia", vec![vec!["j", "a0"]]);
        map.insert("ian", vec![vec!["j", "ɛ0", "n"]]);
        map.insert("iang", vec![vec!["j", "a0", "ŋ"]]);
        map.insert("iao", vec![vec!["j", "au0"]]);
        map.insert("ie", vec![vec!["j", "e0"]]);
        map.insert("in", vec![vec!["i0", "n"]]);
        map.insert("iou", vec![vec!["j", "ou0"]]);
        map.insert("ing", vec![vec!["i0", "ŋ"]]);
        map.insert("iong", vec![vec!["j", "ʊ0", "ŋ"]]);
        map.insert("ong", vec![vec!["ʊ0", "ŋ"]]);
        map.insert("ou", vec![vec!["ou0"]]);
        map.insert("u", vec![vec!["u0"]]);
        map.insert("uei", vec![vec!["w", "ei0"]]);
        map.insert("ua", vec![vec!["w", "a0"]]);
        map.insert("uai", vec![vec!["w", "ai0"]]);
        map.insert("uan", vec![vec!["w", "a0", "n"]]);
        map.insert("uen", vec![vec!["w", "ə0", "n"]]);
        map.insert("uang", vec![vec!["w", "a0", "ŋ"]]);
        map.insert("ueng", vec![vec!["w", "ə0", "ŋ"]]);
        map.insert("ui", vec![vec!["w", "ei0"]]);
        map.insert("un", vec![vec!["w", "ə0", "n"]]);
        map.insert("uo", vec![vec!["w", "o0"]]);
        map.insert("o", vec![vec!["w", "o0"]]); // 注意：这里'o'的映射可能与预期不符，根据注释可能需要特殊处理
        map.insert("ü", vec![vec!["y0"]]);
        map.insert("üe", vec![vec!["ɥ", "e0"]]);
        map.insert("üan", vec![vec!["ɥ", "ɛ0", "n"]]);
        map.insert("ün", vec![vec!["y0", "n"]]);
        map
    });

static FINAL_MAPPING_AFTER_ZH_CH_SH_R: LazyLock<HashMap<&'static str, Vec<Vec<&'static str>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("i", vec![vec!["ɻ0"], vec!["ʐ0"]]);
        map
    });

static FINAL_MAPPING_AFTER_Z_C_S: LazyLock<HashMap<&'static str, Vec<Vec<&'static str>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        map.insert("i", vec![vec!["ɹ0"], vec!["z0"]]);
        map
    });

static TONE_MAPPING: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1u8, "˥");
    map.insert(2u8, "˧˥");
    map.insert(3u8, "˧˩˧");
    map.insert(4u8, "˥˩");
    map.insert(5u8, "");
    map
});

pub(crate) fn split_tone(pinyin: &str) -> (&str, u8) {
    if let Some(t) = pinyin
        .chars()
        .last()
        .and_then(|c| c.to_digit(10).map(|n| n as u8))
    {
        return (&pinyin[..pinyin.len() - 1], t);
    }
    (pinyin, 5)
}

/// uen 转换，还原原始的韵母
/// iou，uei，uen前面加声母的时候，写成iu，ui，un。
/// 例如niu(牛)，gui(归)，lun(论)。
fn convert_uen(s: &str) -> String {
    match s.strip_suffix('n') {
        Some(stem) if stem.ends_with(['u', 'ū', 'ú', 'ǔ', 'ù']) => {
            format!("{}en", stem)
        }
        _ => s.to_string(),
    }
}

/// ü 转换，还原原始的韵母
/// ü行的韵母跟声母j，q，x拼的时候，写成ju(居)，qu(区)，xu(虚)， ü上两点也省略；
/// 但是跟声母n，l拼的时候，仍然写成nü(女)，lü(吕)
fn convert_uv(pinyin: &str) -> String {
    let chars = pinyin.chars().collect::<Vec<_>>();

    match chars.as_slice() {
        [
            c @ ('j' | 'q' | 'x'),
            tone @ ('u' | 'ū' | 'ú' | 'ǔ' | 'ù'),
            rest @ ..,
        ] => {
            let new_tone = match tone {
                'u' => 'ü',
                'ū' => 'ǖ',
                'ú' => 'ǘ',
                'ǔ' => 'ǚ',
                'ù' => 'ǜ',
                _ => unreachable!(),
            };
            format!("{}{}{}", c, new_tone, rest.iter().collect::<String>())
        }
        _ => pinyin.to_string(),
    }
}

/// iou 转换，还原原始的韵母
/// iou，uei，uen前面加声母的时候，写成iu，ui，un。
/// 例如niu(牛)，gui(归)，lun(论)。
fn convert_iou(pinyin: &str) -> String {
    let chars = pinyin.chars().collect::<Vec<_>>();

    match chars.as_slice() {
        // 处理 iu 系列
        [.., 'i', u @ ('u' | 'ū' | 'ú' | 'ǔ' | 'ù')] => {
            format!("{}o{}", &pinyin[..pinyin.len() - 1], u)
        }

        // 其他情况保持原样
        _ => pinyin.to_string(),
    }
}

/// uei 转换，还原原始的韵母
/// iou，uei，uen前面加声母的时候，写成iu，ui，un。
/// 例如niu(牛)，gui(归)，lun(论)。
fn convert_uei(pinyin: &str) -> String {
    let chars = pinyin.chars().collect::<Vec<_>>();

    match chars.as_slice() {
        // 处理 ui 系列
        [.., 'u', i @ ('i' | 'ī' | 'í' | 'ǐ' | 'ì')] => {
            format!("{}e{}", &pinyin[..pinyin.len() - 1], i)
        }

        // 其他情况保持原样
        _ => pinyin.to_string(),
    }
}

/// 零声母转换，还原原始的韵母
/// i行的韵母，前面没有声母的时候，写成yi(衣)，ya(呀)，ye(耶)，yao(腰)，you(忧)，yan(烟)，yin(因)，yang(央)，ying(英)，yong(雍)。
/// u行的韵母，前面没有声母的时候，写成wu(乌)，wa(蛙)，wo(窝)，wai(歪)，wei(威)，wan(弯)，wen(温)，wang(汪)，weng(翁)。
/// ü行的韵母，前面没有声母的时候，写成yu(迂)，yue(约)，yuan(冤)，yun(晕)；ü上两点省略。"""
pub(crate) fn convert_zero_consonant(pinyin: &str) -> String {
    let mut buffer = String::with_capacity(pinyin.len() + 2);
    let chars: Vec<char> = pinyin.chars().collect();

    match chars.as_slice() {
        // 处理Y系转换
        ['y', 'u', rest @ ..] => {
            buffer.push('ü');
            buffer.extend(rest.iter());
        }
        ['y', u @ ('ū' | 'ú' | 'ǔ' | 'ù'), rest @ ..] => {
            buffer.push(match u {
                'ū' => 'ǖ', // ü 第一声
                'ú' => 'ǘ', // ü 第二声
                'ǔ' => 'ǚ', // ü 第三声
                'ù' => 'ǜ', // ü 第四声
                _ => unreachable!(),
            });
            buffer.extend(rest.iter());
        }
        ['y', i @ ('i' | 'ī' | 'í' | 'ǐ' | 'ì'), rest @ ..] => {
            buffer.push(*i);
            buffer.extend(rest.iter());
        }
        ['y', rest @ ..] => {
            buffer.push('i');
            buffer.extend(rest);
        }

        // 处理W系转换
        ['w', u @ ('u' | 'ū' | 'ú' | 'ǔ' | 'ù'), rest @ ..] => {
            buffer.push(*u);
            buffer.extend(rest.iter());
        }
        ['w', rest @ ..] => {
            buffer.push('u');
            buffer.extend(rest);
        }

        // 无需转换的情况
        _ => return pinyin.to_string(),
    }

    // 有效性验证
    if VALID_FINALS.contains(&buffer.as_str()) {
        buffer
    } else {
        pinyin.to_string()
    }
}

pub(crate) fn split_initial(pinyin: &str) -> (&'static str, &str) {
    for &initial in &INITIALS {
        if let Some(stripped) = pinyin.strip_prefix(initial) {
            return (initial, stripped);
        }
    }
    ("", pinyin)
}

fn apply_tone(variants: &[Vec<&str>], tone: u8) -> Vec<Vec<String>> {
    let tone_str = TONE_MAPPING.get(&tone).unwrap_or(&"");
    variants
        .iter()
        .map(|v| v.iter().map(|s| s.replace("0", tone_str)).collect())
        .collect()
}

pub fn pinyin_to_ipa(pinyin: &str) -> Result<Vec<Vec<String>>, PinyinError> {
    let (pinyin, tone) = split_tone(pinyin);
    let pinyin = convert_zero_consonant(pinyin);
    let pinyin = convert_uv(&pinyin);
    let pinyin = convert_iou(&pinyin);
    let pinyin = convert_uei(&pinyin);
    let pinyin = convert_uen(&pinyin);

    // 处理特殊成音节辅音和感叹词
    if let Some(ipa) = SYLLABIC_CONSONANT_MAPPINGS.get(pinyin.as_str()) {
        return Ok(apply_tone(ipa, tone)
            .into_iter()
            .map(|i| i.into_iter().collect())
            .collect());
    }
    if let Some(ipa) = INTERJECTION_MAPPINGS.get(pinyin.as_str()) {
        return Ok(apply_tone(ipa, tone)
            .into_iter()
            .map(|i| i.into_iter().collect())
            .collect());
    }

    // 分解声母韵母
    let (initial_part, final_part) = split_initial(pinyin.as_str());

    // 获取韵母IPA
    let final_ipa = match initial_part {
        "zh" | "ch" | "sh" | "r" if FINAL_MAPPING_AFTER_ZH_CH_SH_R.contains_key(final_part) => {
            FINAL_MAPPING_AFTER_ZH_CH_SH_R.get(final_part)
        }
        "z" | "c" | "s" if FINAL_MAPPING_AFTER_Z_C_S.contains_key(final_part) => {
            FINAL_MAPPING_AFTER_Z_C_S.get(final_part)
        }
        _ => FINAL_MAPPING.get(final_part),
    }
    .ok_or(PinyinError::FinalNotFound(final_part.to_owned()))?;

    // 组合所有可能
    let mut result = Vec::<Vec<String>>::new();
    let initials = INITIAL_MAPPING
        .get(initial_part)
        .map_or(vec![vec![Default::default()]], |i| {
            i.iter()
                .map(|i| i.iter().map(|i| i.to_string()).collect())
                .collect()
        });

    for i in initials.into_iter() {
        for j in apply_tone(final_ipa, tone).into_iter() {
            result.push(
                i.iter()
                    .chain(j.iter())
                    .map(|i| i.to_owned())
                    .collect::<Vec<_>>(),
            )
        }
    }

    Ok(result)
}

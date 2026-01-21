/// 参考了python的misaki库的zh_frontend.py。
use {
    crate::{split_initial, split_tone},
    chinese_number::{ChineseCountMethod, ChineseToNumber},
    jieba_rs::Jieba,
    pinyin::ToPinyin,
    std::{collections::HashMap, sync::LazyLock},
};

const BU: &str = "不";
const YI: &str = "一";
const X_ENG: [&str; 2] = ["x", "eng"];
const PUNC: &str = ";: ,.!?—…\"()“”";
const MUST_NOT_NEURAL_TONE_WORDS: [&str; 40] = [
    "男子",
    "女子",
    "分子",
    "原子",
    "量子",
    "莲子",
    "石子",
    "瓜子",
    "电子",
    "人人",
    "虎虎",
    "幺幺",
    "干嘛",
    "学子",
    "哈哈",
    "数数",
    "袅袅",
    "局地",
    "以下",
    "娃哈哈",
    "花花草草",
    "留得",
    "耕地",
    "想想",
    "熙熙",
    "攘攘",
    "卵子",
    "死死",
    "冉冉",
    "恳恳",
    "佼佼",
    "吵吵",
    "打打",
    "考考",
    "整整",
    "莘莘",
    "落地",
    "算子",
    "家家户户",
    "青青",
];
const MUST_NEURAL_TONE_WORDS: [&str; 417] = [
    "麻烦", "麻利", "鸳鸯", "高粱", "骨头", "骆驼", "马虎", "首饰", "馒头", "馄饨", "风筝", "难为",
    "队伍", "阔气", "闺女", "门道", "锄头", "铺盖", "铃铛", "铁匠", "钥匙", "里脊", "里头", "部分",
    "那么", "道士", "造化", "迷糊", "连累", "这么", "这个", "运气", "过去", "软和", "转悠", "踏实",
    "跳蚤", "跟头", "趔趄", "财主", "豆腐", "讲究", "记性", "记号", "认识", "规矩", "见识", "裁缝",
    "补丁", "衣裳", "衣服", "衙门", "街坊", "行李", "行当", "蛤蟆", "蘑菇", "薄荷", "葫芦", "葡萄",
    "萝卜", "荸荠", "苗条", "苗头", "苍蝇", "芝麻", "舒服", "舒坦", "舌头", "自在", "膏药", "脾气",
    "脑袋", "脊梁", "能耐", "胳膊", "胭脂", "胡萝", "胡琴", "胡同", "聪明", "耽误", "耽搁", "耷拉",
    "耳朵", "老爷", "老实", "老婆", "戏弄", "将军", "翻腾", "罗嗦", "罐头", "编辑", "结实", "红火",
    "累赘", "糨糊", "糊涂", "精神", "粮食", "簸箕", "篱笆", "算计", "算盘", "答应", "笤帚", "笑语",
    "笑话", "窟窿", "窝囊", "窗户", "稳当", "稀罕", "称呼", "秧歌", "秀气", "秀才", "福气", "祖宗",
    "砚台", "码头", "石榴", "石头", "石匠", "知识", "眼睛", "眯缝", "眨巴", "眉毛", "相声", "盘算",
    "白净", "痢疾", "痛快", "疟疾", "疙瘩", "疏忽", "畜生", "生意", "甘蔗", "琵琶", "琢磨", "琉璃",
    "玻璃", "玫瑰", "玄乎", "狐狸", "状元", "特务", "牲口", "牙碜", "牌楼", "爽快", "爱人", "热闹",
    "烧饼", "烟筒", "烂糊", "点心", "炊帚", "灯笼", "火候", "漂亮", "滑溜", "溜达", "温和", "清楚",
    "消息", "浪头", "活泼", "比方", "正经", "欺负", "模糊", "槟榔", "棺材", "棒槌", "棉花", "核桃",
    "栅栏", "柴火", "架势", "枕头", "枇杷", "机灵", "本事", "木头", "木匠", "朋友", "月饼", "月亮",
    "暖和", "明白", "时候", "新鲜", "故事", "收拾", "收成", "提防", "挖苦", "挑剔", "指甲", "指头",
    "拾掇", "拳头", "拨弄", "招牌", "招呼", "抬举", "护士", "折腾", "扫帚", "打量", "打算", "打扮",
    "打听", "打发", "扎实", "扁担", "戒指", "懒得", "意识", "意思", "悟性", "怪物", "思量", "怎么",
    "念头", "念叨", "别人", "快活", "忙活", "志气", "心思", "得罪", "张罗", "弟兄", "开通", "应酬",
    "庄稼", "干事", "帮手", "帐篷", "希罕", "师父", "师傅", "巴结", "巴掌", "差事", "工夫", "岁数",
    "屁股", "尾巴", "少爷", "小气", "小伙", "将就", "对头", "对付", "寡妇", "家伙", "客气", "实在",
    "官司", "学问", "字号", "嫁妆", "媳妇", "媒人", "婆家", "娘家", "委屈", "姑娘", "姐夫", "妯娌",
    "妥当", "妖精", "奴才", "女婿", "头发", "太阳", "大爷", "大方", "大意", "大夫", "多少", "多么",
    "外甥", "壮实", "地道", "地方", "在乎", "困难", "嘴巴", "嘱咐", "嘟囔", "嘀咕", "喜欢", "喇嘛",
    "喇叭", "商量", "唾沫", "哑巴", "哈欠", "哆嗦", "咳嗽", "和尚", "告诉", "告示", "含糊", "吓唬",
    "后头", "名字", "名堂", "合同", "吆喝", "叫唤", "口袋", "厚道", "厉害", "千斤", "包袱", "包涵",
    "匀称", "勤快", "动静", "动弹", "功夫", "力气", "前头", "刺猬", "刺激", "别扭", "利落", "利索",
    "利害", "分析", "出息", "凑合", "凉快", "冷战", "冤枉", "冒失", "养活", "关系", "先生", "兄弟",
    "便宜", "使唤", "佩服", "作坊", "体面", "位置", "似的", "伙计", "休息", "什么", "人家", "亲戚",
    "亲家", "交情", "云彩", "事情", "买卖", "主意", "丫头", "丧气", "两口", "东西", "东家", "世故",
    "不由", "下水", "下巴", "上头", "上司", "丈夫", "丈人", "一辈", "那个", "菩萨", "父亲", "母亲",
    "咕噜", "邋遢", "费用", "冤家", "甜头", "介绍", "荒唐", "大人", "泥鳅", "幸福", "熟悉", "计划",
    "扑腾", "蜡烛", "姥爷", "照顾", "喉咙", "吉他", "弄堂", "蚂蚱", "凤凰", "拖沓", "寒碜", "糟蹋",
    "倒腾", "报复", "逻辑", "盘缠", "喽啰", "牢骚", "咖喱", "扫把", "惦记",
];
const MUST_ERHUA: [&str; 9] = [
    "小院儿",
    "胡同儿",
    "范儿",
    "老汉儿",
    "撒欢儿",
    "寻老礼儿",
    "妥妥儿",
    "媳妇儿",
    "老头儿",
];
const NOT_ERHUA: [&str; 44] = [
    "虐儿",
    "为儿",
    "护儿",
    "瞒儿",
    "救儿",
    "替儿",
    "有儿",
    "一儿",
    "我儿",
    "俺儿",
    "妻儿",
    "拐儿",
    "聋儿",
    "乞儿",
    "患儿",
    "幼儿",
    "孤儿",
    "婴儿",
    "婴幼儿",
    "连体儿",
    "脑瘫儿",
    "流浪儿",
    "体弱儿",
    "混血儿",
    "蜜雪儿",
    "舫儿",
    "祖儿",
    "美儿",
    "应采儿",
    "可儿",
    "侄儿",
    "孙儿",
    "侄孙儿",
    "女儿",
    "男儿",
    "红孩儿",
    "花儿",
    "虫儿",
    "马儿",
    "鸟儿",
    "猪儿",
    "猫儿",
    "狗儿",
    "少儿",
];
const UNK: &str = "❓";

static PHRASES_DICT: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    let phrases = include_str!("../../dict/pinyin.dict");
    for line in phrases.lines() {
        let Some((k, v)) = line.trim().split_once(" ") else {
            continue;
        };
        map.insert(k, v);
    }

    map
});

static JIEBA: LazyLock<Jieba> = LazyLock::new(|| {
    let mut jieba = Jieba::new();
    for k in PHRASES_DICT.keys() {
        jieba.add_word(k, None, Some("x"));
    }

    jieba
});
static ZH_MAP: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("b", "ㄅ");
    map.insert("p", "ㄆ");
    map.insert("m", "ㄇ");
    map.insert("f", "ㄈ");
    map.insert("d", "ㄉ");
    map.insert("t", "ㄊ");
    map.insert("n", "ㄋ");
    map.insert("l", "ㄌ");
    map.insert("g", "ㄍ");
    map.insert("k", "ㄎ");
    map.insert("h", "ㄏ");
    map.insert("j", "ㄐ");
    map.insert("q", "ㄑ");
    map.insert("x", "ㄒ");
    map.insert("zh", "ㄓ");
    map.insert("ch", "ㄔ");
    map.insert("sh", "ㄕ");
    map.insert("r", "ㄖ");
    map.insert("z", "ㄗ");
    map.insert("c", "ㄘ");
    map.insert("s", "ㄙ");
    map.insert("a", "ㄚ");
    map.insert("o", "ㄛ");
    map.insert("e", "ㄜ");
    map.insert("ie", "ㄝ");
    map.insert("ai", "ㄞ");
    map.insert("ei", "ㄟ");
    map.insert("ao", "ㄠ");
    map.insert("ou", "ㄡ");
    map.insert("an", "ㄢ");
    map.insert("en", "ㄣ");
    map.insert("ang", "ㄤ");
    map.insert("eng", "ㄥ");
    map.insert("er", "ㄦ");
    map.insert("i", "ㄧ");
    map.insert("u", "ㄨ");
    map.insert("v", "ㄩ");
    map.insert("ii", "ㄭ");
    map.insert("iii", "十");
    map.insert("ve", "月");
    map.insert("ia", "压");
    map.insert("ian", "言");
    map.insert("iang", "阳");
    map.insert("iao", "要");
    map.insert("in", "阴");
    map.insert("ing", "应");
    map.insert("iong", "用");
    map.insert("iou", "又");
    map.insert("ong", "中");
    map.insert("ua", "穵");
    map.insert("uai", "外");
    map.insert("uan", "万");
    map.insert("uang", "王");
    map.insert("uei", "为");
    map.insert("uen", "文");
    map.insert("ueng", "瓮");
    map.insert("uo", "我");
    map.insert("van", "元");
    map.insert("vn", "云");

    map.insert(";", ";");
    map.insert(":", ":");
    map.insert(",", ",");
    map.insert(".", ".");
    map.insert("!", "!");
    map.insert("?", "?");
    map.insert("/", "/");
    map.insert("—", "—");
    map.insert("…", "…");
    map.insert("\"", "\"");
    map.insert("(", "(");
    map.insert(")", ")");
    map.insert("“", "“");
    map.insert("”", "”");
    map.insert(" ", " ");
    map.insert("1", "1");
    map.insert("2", "2");
    map.insert("3", "3");
    map.insert("4", "4");
    map.insert("5", "5");
    map.insert("R", "R");

    map
});

/// merge "不" and the word behind it
/// if don't merge, "不" sometimes appears alone according to jieba, which may occur sandhi error
fn merge_bu(seg: &mut Vec<(String, String)>) {
    let mut i = 0;
    while i < seg.len() {
        let (left, right) = seg.split_at_mut(i);
        let (word, pos) = &mut right[0];
        if !X_ENG.contains(&pos.as_str()) && i > 0 {
            let last_word = &left[i - 1].0;
            if last_word == BU {
                *word = BU.to_owned() + word;
                seg.remove(i - 1);
                i -= 1; // Adjust index after removal
            }
        }
        i += 1;
    }
}

/// function 1: merge "一" and reduplication words in it's left and right, e.g. "听","一","听" ->"听一听"
/// function 2: merge single  "一" and the word behind it
/// if don't merge, "一" sometimes appears alone according to jieba, which may occur sandhi error
/// e.g.
/// input seg: [('听', 'v'), ('一', 'm'), ('听', 'v')]
/// output seg: [['听一听', 'v']]
fn merge_yi(seg: &mut Vec<(String, String)>) {
    let mut i = 0;
    // function 1
    while i < seg.len() {
        let word = &seg[i].0;
        if i > 0
            && word == YI
            && i + 1 < seg.len()
            && seg[i - 1].0 == seg[i + 1].0
            && seg[i - 1].1 == "v"
            && !X_ENG.contains(&seg[i + 1].1.as_str())
        {
            seg[i - 1].0 = seg[i - 1].0.to_owned() + YI + &seg[i + 1].0;
            seg.remove(i);
            seg.remove(i);
            i -= 1;
        }
        i += 1;
    }

    // function 2
    i = 1;
    while i < seg.len() {
        let (left, right) = seg.split_at_mut(i);
        let (word, pos) = &right[0];
        if left[i - 1].0 == YI && !X_ENG.contains(&pos.as_str()) {
            left[i - 1].0 += word;
            seg.remove(i);
            i -= 1;
        }
        i += 1;
    }
}

fn merge_reduplication(seg: &mut Vec<(String, String)>) {
    let mut i = 1;
    while i < seg.len() {
        let (left, right) = seg.split_at_mut(i);
        let (word, pos) = &right[0];
        if word == &left[i - 1].0 && !X_ENG.contains(&pos.as_str()) {
            left[i - 1].0.push_str(word);
            seg.remove(i);
            i -= 1;
        }
        i += 1;
    }
}

fn is_reduplication(word: &str) -> bool {
    if word.len() != 2 {
        false
    } else {
        let mut word = word.chars();
        word.next() == word.next()
    }
}

fn get_pinyin(word: &str) -> Vec<&'static str> {
    word.chars()
        .filter_map(|i| i.to_pinyin().map(|i| i.with_tone_num_end()))
        .collect::<Vec<_>>()
}

/// the first and the second words are all_tone_three
fn merge_continuous_three_tones(seg: &mut Vec<(String, String)>) {
    let mut pinyin_list = vec![vec!["0"]; seg.len()];
    for (i, (word, pos)) in seg.iter().enumerate() {
        if X_ENG.contains(&pos.as_str()) {
            continue;
        }

        pinyin_list[i] = get_pinyin(word);
    }

    let mut merge_last = vec![false; seg.len()];
    let mut i = 1;
    while i < seg.len() {
        let (left, right) = seg.split_at_mut(i);
        let (word, pos) = &right[0];
        if !X_ENG.contains(&pos.as_str())
            && pinyin_list[i - 1].iter().all(|i| i.ends_with("3"))
            && pinyin_list[i].iter().all(|i| i.ends_with("3"))
            && !merge_last[i - 1]
            && !is_reduplication(&left[i - 1].0)
            && left[i - 1].0.chars().count() + word.chars().count() <= 3
        {
            merge_last[i] = true;
            left[i - 1].0 += word;
            seg.remove(i);
            i -= 1;
        }
        i += 1;
    }
}

/// the last char of first word and the first char of second word is tone_three
fn merge_continuous_three_tones_2(seg: &mut Vec<(String, String)>) {
    let mut pinyin_list = vec![vec!["0"]; seg.len()];
    for (i, (word, pos)) in seg.iter().enumerate() {
        if X_ENG.contains(&pos.as_str()) {
            continue;
        }

        pinyin_list[i] = get_pinyin(word);
    }

    let mut merge_last = vec![false; seg.len()];
    let mut i = 1;
    while i < seg.len() {
        let (left, right) = seg.split_at_mut(i);
        let (word, pos) = &right[0];
        if !X_ENG.contains(&pos.as_str())
            && pinyin_list[i - 1].last().is_some_and(|i| i.ends_with("3"))
            && pinyin_list[i].first().is_some_and(|i| i.ends_with("3"))
            && !merge_last[i - 1]
            && !is_reduplication(&left[i - 1].0)
            && left[i - 1].0.chars().count() + word.chars().count() <= 3
        {
            merge_last[i] = true;
            left[i - 1].0 += word;
            seg.remove(i);
            i -= 1;
        }
        i += 1;
    }
}

fn merge_er(seg: &mut Vec<(String, String)>) {
    let mut i = 1;
    while i < seg.len() {
        let (left, right) = seg.split_at_mut(i);
        let word = &right[0].0;
        if word == "儿" && !X_ENG.contains(&left[i - 1].1.as_str()) {
            left[i - 1].0 += word;
            seg.remove(i);
            i -= 1;
        }
        i += 1;
    }
}

fn pre_merge_for_modify(seg: &mut Vec<(String, String)>) {
    merge_bu(seg);
    merge_yi(seg);
    merge_reduplication(seg);
    merge_continuous_three_tones(seg);
    merge_continuous_three_tones_2(seg);
    merge_er(seg);
}

fn bu_sandhi(word: &str, pinyins: &mut [String]) {
    let len = word.chars().count();
    // e.g. 看不懂
    if len == 3 && word.chars().nth(1) == BU.chars().next() {
        if let Some(i) = pinyins.get_mut(1) {
            i.pop();
            i.push('5');
        }
    } else {
        for (i, ch) in word.chars().enumerate() {
            // "不" before tone4 should be bu2, e.g. 不怕
            if BU.starts_with(ch)
                && i + 1 < len
                && pinyins[i + 1].ends_with("4")
                && let Some(i) = pinyins.get_mut(1)
            {
                i.pop();
                i.push('2');
            }
        }
    }
}

fn yi_sandhi(word: &str, pinyins: &mut [String]) {
    // "一" in number sequences, e.g. 一零零, 二一零
    if word.find(YI).is_some()
        && (word
            .chars()
            .filter(|c| !YI.starts_with(*c))
            .all(char::is_numeric)
            || ChineseToNumber::<i32>::to_number(&word, ChineseCountMethod::TenThousand).is_ok())
    {
    }
    // "一" between reduplication words shold be yi5, e.g. 看一看
    else if word.chars().count() == 3
        && word.chars().nth(1) == YI.chars().next()
        && word.chars().next() == word.chars().next_back()
    {
        if let Some(i) = pinyins.get_mut(1) {
            i.pop();
            i.push('5');
        }
    }
    // when "一" is ordinal word, it should be yi1
    else if word.starts_with("第一") {
        if let Some(i) = pinyins.get_mut(1) {
            i.pop();
            i.push('1');
        }
    } else {
        for (i, ch) in word.chars().enumerate() {
            if YI.starts_with(ch) && i + 1 < word.chars().count() {
                // "一" before tone4 should be yi2, e.g. 一段
                if pinyins[i + 1]
                    .chars()
                    .next_back()
                    .map(|c| c == '4' || c == '5')
                    .unwrap_or_default()
                {
                    if let Some(i) = pinyins.get_mut(i) {
                        i.pop();
                        i.push('2');
                    }
                }
                // "一" before non-tone4 should be yi4, e.g. 一天
                else {
                    // "一" 后面如果是标点，还读一声
                    if word
                        .chars()
                        .nth(i + 1)
                        .map(|c| !PUNC.contains(c))
                        .unwrap_or_default()
                        && let Some(i) = pinyins.get_mut(i)
                    {
                        i.pop();
                        i.push('4');
                    }
                }
            }
        }
    }
}

fn split_at_char(s: &str, mid: usize) -> (&str, &str) {
    let mut chars = s.char_indices();
    let byte_pos = chars.nth(mid).map(|(i, _)| i).unwrap_or(s.len());
    s.split_at(byte_pos)
}

fn split_word(word: &str) -> (String, String) {
    let mut word_list = JIEBA.cut_for_search(word, true);
    word_list.sort_by_cached_key(|i| i.chars().count());
    let first_subword = &word_list[0];
    if let Some(0) = word.find(first_subword) {
        let (_, second_subword) = word.split_at(first_subword.len());
        ((*first_subword).into(), second_subword.into())
    } else {
        let (second_subword, _) = word.split_at(word.len() - first_subword.len());
        (second_subword.into(), (*first_subword).into())
    }
}

/// the meaning of jieba pos tag: https://blog.csdn.net/weixin_44174352/article/details/113731041
/// e.g.
/// word: "家里"
/// pos: "s"
/// finals: ['ia1', 'i3']
fn neural_sandhi(word: &str, pos: &str, pinyins: &mut [String]) {
    if MUST_NOT_NEURAL_TONE_WORDS.contains(&word) {
        return;
    }

    // reduplication words for n. and v. e.g. 奶奶, 试试, 旺旺
    for (j, item) in word.chars().enumerate() {
        if j >= 1
            && Some(item) == word.chars().nth(j - 1)
            && pos
                .chars()
                .next()
                .map(|c| c == 'n' || c == 'v' || c == 'a')
                .unwrap_or_default()
            && let Some(i) = pinyins.get_mut(j)
        {
            i.pop();
            i.push('5');
        }
    }

    let len = word.chars().count();
    let ge_idx = word
        .chars()
        .enumerate()
        .find(|(_, c)| c == &'个')
        .map(|i| i.0 as isize)
        .unwrap_or(-1);
    if (word
                 .chars()
                 .next_back()
                 .map(|c| "的地得".contains(c))
                 .unwrap_or_default() || word
             .chars()
             .next_back()
             .map(|c| "吧呢啊呐噻嘛吖嗨呐哦哒滴哩哟喽啰耶喔诶".contains(c))
             .unwrap_or_default()) && len >= 1
        // e.g. 走了, 看着, 去过
        || (len == 1 && "了着过".find(word).is_some() && ["ul", "uz", "ug"].contains(&pos))||(len > 1
        && word
        .chars()
        .next_back()
        .map(|c| "们子".contains(c))
        .unwrap_or_default()
        && ["r", "n"].contains(&pos)
    )
        // e.g. 桌上, 地下
        ||(len > 1
        && word
        .chars()
        .next_back()
        .map(|c| "上下".contains(c))
        .unwrap_or_default()
        && ["s", "l", "f"].contains(&pos)
    )
        // e.g. 上来, 下去
        ||(len > 1
        && word
        .chars()
        .next_back()
        .map(|c| "来去".contains(c))
        .unwrap_or_default()
        && word
        .chars()
        .nth(len - 2)
        .map(|c| "上下进出回过起开".contains(c))
        .unwrap_or_default()
    ) {
        if let Some(i) = pinyins.last_mut() {
            if i.ends_with(char::is_numeric) {
                i.pop();
            }
            i.push('5');
        }
    }
    // 个做量词
    else if (ge_idx >= 1
        && (ChineseToNumber::<i32>::to_number(
            &split_at_char(word, ge_idx as _).0,
            ChineseCountMethod::TenThousand,
        )
        .is_ok()
            || word
                .chars()
                .nth((ge_idx - 1) as _)
                .map(|c| "几有两半多各整每做是".contains(c))
                .unwrap_or_default()))
        || word == "个"
    {
        if let Some(i) = pinyins.get_mut(ge_idx as usize) {
            i.pop();
            let _: () = i.push('5');
        }
    } else {
        if (MUST_NEURAL_TONE_WORDS.contains(&word)
            || (len >= 2 && MUST_NEURAL_TONE_WORDS.contains(&split_at_char(word, len - 2).1)))
            && let Some(i) = pinyins.last_mut()
        {
            i.pop();
            i.push('5');
        }
    }

    let (left_word, right_word) = split_word(word);
    let (left_pinyins, right_pinyins) = pinyins.split_at_mut(left_word.chars().count());

    // conventional neural in Chinese
    let len = left_word.chars().count();
    if (MUST_NEURAL_TONE_WORDS.contains(&left_word.as_str())
        || (len >= 2 && MUST_NEURAL_TONE_WORDS.contains(&split_at_char(&left_word, len - 2).1)))
        && let Some(i) = left_pinyins.last_mut()
    {
        i.pop();
        i.push('5');
    }
    let len = right_word.chars().count();
    if (MUST_NEURAL_TONE_WORDS.contains(&right_word.as_str())
        || (len >= 2 && MUST_NEURAL_TONE_WORDS.contains(&split_at_char(&right_word, len - 2).1)))
        && let Some(i) = right_pinyins.last_mut()
    {
        i.pop();
        i.push('5');
    }
}

fn three_sandhi(word: &str, pinyins: &mut [String]) {
    let len = word.chars().count();
    if len == 2 && pinyins.iter().all(|i| i.ends_with("3")) {
        if let Some(i) = pinyins.first_mut() {
            i.pop();
            i.push('2');
        }
    } else if len == 3 {
        let (left_word, _) = split_word(word);
        if pinyins.iter().all(|i| i.ends_with("3")) {
            //  disyllabic + monosyllabic, e.g. 蒙古/包
            if left_word.chars().count() == 2 {
                if let Some(i) = pinyins.first_mut() {
                    i.pop();
                    let _: () = i.push('2');
                }
                if let Some(i) = pinyins.get_mut(1) {
                    i.pop();
                    i.push('2');
                }
            }
            //  monosyllabic + disyllabic, e.g. 纸/老虎
            else if left_word.chars().count() == 1
                && let Some(i) = pinyins.get_mut(1)
            {
                i.pop();
                i.push('2');
            }
        } else {
            let (left_pinyins, right_pinyins) = pinyins.split_at_mut(left_word.chars().count());
            // e.g. 所有/人
            if left_pinyins.iter().all(|i| i.ends_with("3"))
                && left_pinyins.len() == 2
                && let Some(i) = left_pinyins.first_mut()
            {
                i.pop();
                i.push('2');
            }
            if right_pinyins.iter().all(|i| i.ends_with("3"))
                && right_pinyins.len() == 2
                && let Some(i) = right_pinyins.first_mut()
            {
                i.pop();
                i.push('2');
            }
            // e.g. 好/喜欢
            if !right_pinyins.iter().all(|i| i.ends_with("3"))
                && right_pinyins
                    .first()
                    .map(|i| i.ends_with("3"))
                    .unwrap_or_default()
                && left_pinyins
                    .last()
                    .map(|i| i.ends_with("3"))
                    .unwrap_or_default()
                && let Some(i) = left_pinyins.last_mut()
            {
                i.pop();
                let _: () = i.push('2');
            }
        }
    }
    // split idiom into two words who's length is 2
    else if len == 4 {
        let (left_pinyins, right_pinyins) = pinyins.split_at_mut(2);
        if left_pinyins.iter().all(|i| i.ends_with("3"))
            && let Some(i) = left_pinyins.first_mut()
        {
            i.pop();
            i.push('2');
        }
        if right_pinyins.iter().all(|i| i.ends_with("3"))
            && let Some(i) = right_pinyins.first_mut()
        {
            i.pop();
            i.push('2');
        }
    }
}

fn get_pinyin_fine(word: &str) -> Vec<String> {
    let mut pinyin = if PHRASES_DICT.contains_key(word) {
        PHRASES_DICT
            .get(word)
            .map(|p| p.split(' ').map(|i| i.to_owned()).collect::<Vec<_>>())
            .unwrap_or_default()
    } else {
        get_pinyin(word)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    };

    for p in pinyin.iter_mut() {
        let Some(tone) = p.chars().next_back() else {
            continue;
        };
        if !tone.is_numeric() {
            continue;
        }

        // 处理整体认读音节
        if p.starts_with("zi") || p.starts_with("ci") || p.starts_with("si") {
            p.pop();
            p.push('i');
            p.push(tone);
        } else if p.starts_with("ri")
            || p.starts_with("zhi")
            || p.starts_with("chi")
            || p.starts_with("shi")
        {
            p.pop();
            p.push('i');
            p.push('i');
            p.push(tone);
        }
    }
    pinyin
}

/// * `word`: 分词
/// * `pos`: 词性
/// * `pinyins`: 带调拼音, [pinyin1, ..., pinyinN]
fn modified_tone(word: &str, pos: &str, pinyins: &mut [String]) {
    bu_sandhi(word, pinyins);
    yi_sandhi(word, pinyins);
    neural_sandhi(word, pos, pinyins);
    three_sandhi(word, pinyins);
}

fn merge_erhua(word: &str, pos: &str, pinyins: &mut [String]) {
    // fix er1
    let mut i = 0;
    while i < pinyins.len() {
        if i == pinyins.len() - 1
            && word.chars().nth(i).map(|c| c == '儿').unwrap_or_default()
            && pinyins[i].ends_with("er1")
            && let Some(i) = pinyins.get_mut(i)
        {
            let _: () = *i = "er2".to_owned();
        }

        i += 1;
    }

    // 发音
    if !MUST_ERHUA.contains(&word) && (NOT_ERHUA.contains(&word) || ["a", "j", "nr"].contains(&pos))
    {
        return;
    }

    // "……" 等情况直接返回
    if pinyins.len() != word.chars().count() {
        return;
    }

    // 不发音
    i = 1;
    while i < pinyins.len() {
        if i == pinyins.len() - 1
            && word.chars().nth(i).map(|c| c == '儿').unwrap_or_default()
            && ["er2", "er5"].contains(&pinyins[i].as_str())
            && !NOT_ERHUA.contains(&split_at_char(word, word.chars().count() - 2).1)
        {
            pinyins.last_mut().and_then(|i| {
                let c = i.pop()?;
                i.push('R');
                let _: () = i.push(c);
                Some(())
            });
        }
        i += 1;
    }
}

/// Return: string of phonemes.
/// 'ㄋㄧ2ㄏㄠ3/ㄕ十4ㄐㄝ4'
pub(super) fn g2p(text: &str, with_erhua: bool) -> String {
    let mut seg_cut = JIEBA
        .tag(text, true)
        .iter()
        .map(|i| (i.word.to_string(), i.tag.to_string()))
        .collect::<Vec<_>>();

    // fix wordseg bad case for sandhi
    pre_merge_for_modify(&mut seg_cut);

    struct MToken {
        tag: String,
        phonemes: String,
        whitespace: String,
    }

    // 为了多音词获得更好的效果，这里采用整句预测
    let mut tokens = Vec::with_capacity(seg_cut.len());
    // pypinyin, g2pM
    for (word, pos) in seg_cut.iter() {
        let tag = if pos == "x"
            && word
                .chars()
                .min()
                .map(|c| '\u{4E00}' <= c)
                .unwrap_or_default()
            && word
                .chars()
                .max()
                .map(|c| c <= '\u{9FFF}')
                .unwrap_or_default()
        {
            "X".into()
        } else if pos != "x" && PUNC.contains(word) {
            "x".into()
        } else {
            pos.to_owned()
        };
        let mut tk = MToken {
            tag,
            whitespace: Default::default(),
            phonemes: Default::default(),
        };
        if X_ENG.contains(&tk.tag.as_str()) {
            if !word.trim().is_empty() {
                if tk.tag == "x" && PUNC.contains(word) {
                    tk.phonemes = word.to_owned();
                }
                tokens.push(tk);
            } else if !tokens.is_empty()
                && let Some(i) = tokens.last_mut()
            {
                let _: () = i.whitespace += word;
            }
            continue;
        } else if !tokens.is_empty()
            && tokens
                .last()
                .map(|i| !X_ENG.contains(&i.tag.as_str()))
                .unwrap_or_default()
            && tokens
                .last()
                .map(|i| i.whitespace.is_empty())
                .unwrap_or_default()
            && let Some(i) = tokens.last_mut()
        {
            let _: () = i.whitespace = "/".to_owned();
        }

        // g2p
        let mut pinyins = get_pinyin_fine(word);
        // tone sandhi
        modified_tone(word, pos, &mut pinyins);
        // er hua
        if with_erhua {
            merge_erhua(word, pos, &mut pinyins);
        }

        let mut phones = Vec::with_capacity(pinyins.len());
        for p in pinyins.iter() {
            // NOTE: post process for pypinyin outputs
            // we discriminate i, ii and iii
            let (c, v) = split_initial(p);
            let mut v = v.to_owned();
            convert_pinyin(c, &mut v);
            let (f, t) = split_tone(v.as_str());
            if !c.is_empty() {
                phones.push(c.to_owned());
            }
            // replace punctuation by ` `
            if !v.is_empty() {
                // and v not in rhy_phns:
                if !PUNC.contains(v.as_str()) {
                    phones.push(f.to_owned());
                    phones.push(t.to_string());
                } else if v != c {
                    phones.push(v.to_owned());
                }
            }
        }
        let phones = phones.join("_").replace("_eR", "_er").replace('R', "_R");
        tk.phonemes = phones
            .split('_')
            .map(|c| *ZH_MAP.get(c).unwrap_or(&UNK))
            .collect::<String>();
        tokens.push(tk);
    }

    tokens
        .iter()
        .map(|tk| {
            if tk.phonemes.is_empty() {
                return UNK.to_owned() + &tk.whitespace;
            }
            tk.phonemes.to_owned() + &tk.whitespace
        })
        .collect()
}

fn convert_pinyin(initial_part: &str, final_part: &mut String) {
    let chars = final_part.chars().collect::<Vec<_>>();
    // 先替换
    if let Some(i) = final_part.find('ü') {
        final_part.replace_range(i..i + 2, "v");
    }
    match chars.as_slice() {
        ['u', ..] if initial_part.starts_with(['j', 'q', 'x']) => {
            final_part.replace_range(0..1, "v")
        }
        ['y', 'u', ..] => final_part.replace_range(0..2, "v"),
        ['y', 'i', ..] => final_part.replace_range(0..2, "i"),
        ['y', ..] => final_part.replace_range(0..1, "i"),
        ['w', 'u', ..] => final_part.replace_range(0..2, "u"),
        ['w', ..] => final_part.replace_range(0..1, "u"),
        _ => (),
    }

    // 第二步：还原简写韵母
    const ABBREVIATIONS: [(&str, &str); 3] = [("iu", "iou"), ("ui", "uei"), ("un", "uen")];
    for (short, long) in ABBREVIATIONS {
        if let Some(i) = final_part.find(short) {
            final_part.replace_range(i..i + short.len(), long);
            break; // 每个韵母只处理一个简写
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jieba_rs::Jieba;

    #[test]
    fn test_merge_bu() {
        let jieba = Jieba::new();
        let mut seg = jieba
            .tag("不好看", true)
            .iter()
            .map(|i| (i.word.to_string(), i.tag.to_string()))
            .collect::<Vec<_>>();
        assert_eq!(
            vec![("不".into(), "d".into()), ("好看".into(), "v".into())],
            seg
        );
        merge_bu(&mut seg);
        assert_eq!(vec![("不好看".into(), "v".into())], seg);
    }

    #[test]
    fn test_merge_yi() {
        let jieba = Jieba::new();
        let mut seg = jieba
            .tag("听一听一个", true)
            .iter()
            .map(|i| (i.word.to_string(), i.tag.to_string()))
            .collect::<Vec<_>>();
        assert_eq!(
            vec![
                ("听".into(), "v".into()),
                ("一".into(), "m".into()),
                ("听".into(), "v".into()),
                ("一个".into(), "m".into())
            ],
            seg
        );
        merge_yi(&mut seg);
        assert_eq!(
            vec![("听一听".into(), "v".into()), ("一个".into(), "m".into())],
            seg
        );
    }

    #[test]
    fn test_merge_reduplication() {
        let jieba = Jieba::new();
        let mut seg = jieba
            .tag("谢谢谢谢", true)
            .iter()
            .map(|i| (i.word.to_string(), i.tag.to_string()))
            .collect::<Vec<_>>();

        assert_eq!(
            vec![("谢谢".into(), "nr".into()), ("谢谢".into(), "nr".into())],
            seg
        );
        merge_reduplication(&mut seg);
        assert_eq!(vec![("谢谢谢谢".into(), "nr".into())], seg);
    }

    #[test]
    fn test_merge_continuous_three_tones() {
        let jieba = Jieba::new();
        let mut seg = jieba
            .tag("小美好", true)
            .iter()
            .map(|i| (i.word.to_string(), i.tag.to_string()))
            .collect::<Vec<_>>();

        assert_eq!(
            vec![("小".into(), "a".into()), ("美好".into(), "a".into()),],
            seg
        );
        merge_continuous_three_tones(&mut seg);
        assert_eq!(vec![("小美好".into(), "a".into())], seg);
    }

    #[test]
    fn test_merge_continuous_three_tones_2() {
        let jieba = Jieba::new();
        let mut seg = jieba
            .tag("风景好", true)
            .iter()
            .map(|i| (i.word.to_string(), i.tag.to_string()))
            .collect::<Vec<_>>();

        assert_eq!(
            vec![("风景".into(), "n".into()), ("好".into(), "a".into()),],
            seg
        );
        merge_continuous_three_tones_2(&mut seg);
        assert_eq!(vec![("风景好".into(), "n".into())], seg);
    }

    #[test]
    fn test_merge_er() {
        let jieba = Jieba::new();
        let mut seg = jieba
            .tag("红花儿", true)
            .iter()
            .map(|i| (i.word.to_string(), i.tag.to_string()))
            .collect::<Vec<_>>();

        assert_eq!(
            vec![("红花".into(), "n".into()), ("儿".into(), "n".into()),],
            seg
        );
        merge_er(&mut seg);
        assert_eq!(vec![("红花儿".into(), "n".into())], seg);
    }

    #[test]
    fn test_pre_merge_for_modify() {
        let jieba = Jieba::new();
        let mut seg = jieba
            .tag("小宝儿挖一挖", true)
            .iter()
            .map(|i| (i.word.to_string(), i.tag.to_string()))
            .collect::<Vec<_>>();

        assert_eq!(
            vec![
                ("小宝".into(), "nr".into()),
                ("儿".into(), "n".into()),
                ("挖".into(), "v".into()),
                ("一".into(), "m".into()),
                ("挖".into(), "v".into()),
            ],
            seg
        );
        pre_merge_for_modify(&mut seg);
        assert_eq!(
            vec![
                ("小宝儿".into(), "nr".into()),
                ("挖一挖".into(), "v".into())
            ],
            seg
        );
    }

    #[test]
    fn test_bu_sandhi() {
        let mut pinyin = vec!["kan4".into(), "bu4".into(), "dong3".into()];
        bu_sandhi("看不懂", &mut pinyin);
        assert_eq!(
            vec!["kan4".to_string(), "bu5".into(), "dong3".into()],
            pinyin
        );
    }

    #[test]
    fn test_yi_sandhi() {
        let mut pinyin = vec!["yi1".into(), "duan4".into()];
        yi_sandhi("一段", &mut pinyin);
        assert_eq!(vec!["yi2".to_string(), "duan4".into()], pinyin);
        let mut pinyin = vec!["yi1".into(), "bai3".into(), "wan4".into()];
        yi_sandhi("一百万", &mut pinyin);
        assert_eq!(
            vec!["yi1".to_string(), "bai3".into(), "wan4".into()],
            pinyin
        );
        let mut pinyin = vec!["yi1".into(), "tian1".into()];
        yi_sandhi("一天", &mut pinyin);
        assert_eq!(vec!["yi4".to_string(), "tian1".into()], pinyin);
    }

    #[test]
    fn test_neural_sandhi() {
        let mut pinyin = vec!["yi1".into(), "fu2".into()];
        neural_sandhi("衣服", "n", &mut pinyin);
        assert_eq!(vec!["yi1".to_string(), "fu5".into()], pinyin);
        let mut pinyin = vec!["yi1".into(), "ge4".into()];
        neural_sandhi("一个", "m", &mut pinyin);
        assert_eq!(vec!["yi1".to_string(), "ge5".into()], pinyin);
        let mut pinyin = vec!["hu2".into(), "lu2".into()];
        neural_sandhi("葫芦", "n", &mut pinyin);
        assert_eq!(vec!["hu2".to_string(), "lu5".into()], pinyin);
        let mut pinyin = vec!["jian3".to_string(), "dan1".into(), "de".into()];
        neural_sandhi("簡單的", "a", &mut pinyin);
        assert_eq!(
            vec!["jian3".to_string(), "dan1".into(), "de5".into()],
            pinyin
        );
    }

    #[test]
    fn test_three_sandhi() {
        let mut pinyin = vec!["ni3".into(), "hao3".into()];
        three_sandhi("你好", &mut pinyin);
        assert_eq!(vec!["ni2".to_string(), "hao3".into()], pinyin);
        let mut pinyin = vec!["suo3".into(), "you3".into(), "ren2".into()];
        three_sandhi("所有人", &mut pinyin);
        assert_eq!(
            vec!["suo2".to_string(), "you3".into(), "ren2".into()],
            pinyin
        );
        let mut pinyin = vec!["zhu3".into(), "zai3".into()];
        three_sandhi("主宰", &mut pinyin);
        assert_eq!(vec!["zhu2".to_string(), "zai3".into()], pinyin);
    }

    #[test]
    fn test_get_pinyin_fine() {
        let pinyin = get_pinyin_fine("在这里事实上");
        assert_eq!(
            vec![
                "zai4".to_string(),
                "zhe4".into(),
                "li3".into(),
                "shiii4".into(),
                "shiii2".into(),
                "shang4".into()
            ],
            pinyin
        );
    }

    #[test]
    fn test_split_word() {
        let (left, right) = split_word("你好呀");
        assert_eq!(left, "你好");
        assert_eq!(right, "呀");
    }

    #[test]
    fn test_modified_tone() {
        let mut pinyin = vec!["kan4".to_string(), "yi1".into(), "kan4".into()];
        modified_tone("看一看", "v", &mut pinyin);
        assert_eq!(
            vec!["kan4".to_string(), "yi5".into(), "kan4".into()],
            pinyin
        );
    }

    #[test]
    fn test_merge_erhua() {
        let mut pinyin = vec!["lao3".to_string(), "tou2".into(), "er2".into()];
        merge_erhua("老头儿", "n", &mut pinyin);
        assert_eq!(
            vec!["lao3".to_string(), "tou2".into(), "erR2".into()],
            pinyin
        );
    }

    #[test]
    fn test_convert_pinyin() {
        let test_cases = [
            ("j", "uan", "van"), // juan -> jvan
            ("", "yuan", "van"),
            ("", "yue", "ve"),
            ("x", "un", "vn"), // xun -> xvn
            ("k", "un", "uen"),
            ("l", "iu", "iou"), // liu -> liou
            ("", "wu", "u"),
            ("h", "ui", "uei"),
            ("", "wen", "uen"),
        ];

        for (initial, input, expected) in test_cases {
            let mut pinyin = input.to_string();
            convert_pinyin(initial, &mut pinyin);
            assert_eq!(pinyin, expected, "failed: {}+{}", initial, input);
        }
    }

    #[test]
    fn test_g2p() {
        assert_eq!(g2p("借还款", true), "ㄐㄝ4ㄏ万2ㄎ万3");
        assert_eq!(g2p("时间为", true), "ㄕ十2ㄐ言1为2");
    }
}

use crate::{G2PError, pinyin_to_ipa};

fn retone(p: &str) -> String {
    let chars: Vec<char> = p.chars().collect();
    let mut result = String::with_capacity(p.len());
    let mut i = 0;

    while i < chars.len() {
        match () {
            // 三声调优先处理
            _ if i + 2 < chars.len()
                && chars[i] == '˧'
                && chars[i + 1] == '˩'
                && chars[i + 2] == '˧' =>
            {
                result.push('↓');
                i += 3;
            }
            // 二声调
            _ if i + 1 < chars.len() && chars[i] == '˧' && chars[i + 1] == '˥' => {
                result.push('↗');
                i += 2;
            }
            // 四声调
            _ if i + 1 < chars.len() && chars[i] == '˥' && chars[i + 1] == '˩' => {
                result.push('↘');
                i += 2;
            }
            // 一声调
            _ if chars[i] == '˥' => {
                result.push('→');
                i += 1;
            }
            // 组合字符替换（ɻ̩ 和 ɱ̩）
            _ if !(i + 1 >= chars.len() || chars[i+1] != '\u{0329}' || chars[i] != '\u{027B}' && chars[i] != '\u{0271}') =>
            {
                result.push('ɨ');
                i += 2;
            }
            // 默认情况
            _ => {
                result.push(chars[i]);
                i += 1;
            }
        }
    }

    assert!(
        !result.contains('\u{0329}'),
        "Unexpected combining mark in: {}",
        result
    );
    result
}

pub(super) fn py2ipa(py: &str) -> Result<String, G2PError> {
    pinyin_to_ipa(py)?
        .first()
        .map_or(Err(G2PError::EnptyData), |i| {
            Ok(i.iter().map(|i| retone(i)).collect::<String>())
        })
}

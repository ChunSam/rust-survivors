/// Phase 11 로컬라이제이션 — 한국어 / 영어 지원.
///
/// `Lang` 을 `MetaSave.lang` 에 저장, L 키로 토글.
/// 문자열 선택은 `loc(lang, ko, en)` 단일 헬퍼로 통일.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Lang {
    #[default]
    Ko,
    En,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self { Lang::Ko => Lang::En, Lang::En => Lang::Ko }
    }
}

#[inline]
pub fn loc(lang: Lang, ko: &'static str, en: &'static str) -> &'static str {
    match lang { Lang::Ko => ko, Lang::En => en }
}

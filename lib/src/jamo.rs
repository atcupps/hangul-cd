
// Jamo sets
const CONSONANTS: &str = "ㅂㅈㄷㄱㅅㅁㄴㅇㄹㅎㅋㅌㅊㅍ";
const COMPOSITE_CONSONANTS: &str = "ㄲㄸㅃㅆㅉㄵㄺㅄㄳㄶㄻㄼㄽㄾㄿㅀ";
const INITIAL_COMPOSITE_CONSONANTS: &str = "ㄲㄸㅃㅆㅉ";
const FINAL_COMPOSITE_CONSONANTS: &str = "ㄲㄵㄺㅄㅆㄳㄶㄻㄼㄽㄾㄿㅀ";
const VOWELS: &str = "ㅛㅕㅑㅐㅔㅒㅖㅗㅓㅏㅣㅠㅜㅡ";
const COMPOSITE_VOWELS: &str = "ㅘㅙㅚㅝㅞㅟㅢ";

// Jamo arithmetic
pub(crate) const S_BASE: u32 = 0xAC00;
pub(crate) const L_BASE: u32 = 0x1100;
pub(crate) const V_BASE: u32 = 0x1161;
pub(crate) const T_BASE: u32 = 0x11A7;
pub(crate) const V_COUNT: u32 = 21;
pub(crate) const T_COUNT: u32 = 28;
pub(crate) const N_COUNT: u32 = V_COUNT * T_COUNT;

pub(crate) fn create_composite_initial(c1: char, c2: char) -> Option<char> {
    match (c1, c2) {
        ('ㄱ', 'ㄱ') => Some('ㄲ'),
        ('ㄷ', 'ㄷ') => Some('ㄸ'),
        ('ㅂ', 'ㅂ') => Some('ㅃ'),
        ('ㅅ', 'ㅅ') => Some('ㅆ'),
        ('ㅈ', 'ㅈ') => Some('ㅉ'),
        _ => None,
    }
}

pub(crate) fn decompose_composite_initial(c: char) -> Option<(char, char)> {
    match c {
        'ㄲ' => Some(('ㄱ', 'ㄱ')),
        'ㄸ' => Some(('ㄷ', 'ㄷ')),
        'ㅃ' => Some(('ㅂ', 'ㅂ')),
        'ㅆ' => Some(('ㅅ', 'ㅅ')),
        'ㅉ' => Some(('ㅈ', 'ㅈ')),
        _ => None,
    }
}

pub(crate) fn create_composite_vowel(v1: char, v2: char) -> Option<char> {
    match (v1, v2) {
        ('ㅗ', 'ㅏ') => Some('ㅘ'),
        ('ㅗ', 'ㅐ') => Some('ㅙ'),
        ('ㅗ', 'ㅣ') => Some('ㅚ'),
        ('ㅜ', 'ㅓ') => Some('ㅝ'),
        ('ㅜ', 'ㅔ') => Some('ㅞ'),
        ('ㅜ', 'ㅣ') => Some('ㅟ'),
        ('ㅡ', 'ㅣ') => Some('ㅢ'),
        _ => None,
    }
}

pub(crate) fn decompose_composite_vowel(c: char) -> Option<(char, char)> {
    match c {
        'ㅘ' => Some(('ㅗ', 'ㅏ')),
        'ㅙ' => Some(('ㅗ', 'ㅐ')),
        'ㅚ' => Some(('ㅗ', 'ㅣ')),
        'ㅝ' => Some(('ㅜ', 'ㅓ')),
        'ㅞ' => Some(('ㅜ', 'ㅔ')),
        'ㅟ' => Some(('ㅜ', 'ㅣ')),
        'ㅢ' => Some(('ㅡ', 'ㅣ')),
        _ => None,
    }
}

pub(crate) fn create_composite_final(c1: char, c2: char) -> Option<char> {
    match (c1, c2) {
        ('ㄱ', 'ㄱ') => Some('ㄲ'),
        ('ㄴ', 'ㅈ') => Some('ㄵ'),
        ('ㄹ', 'ㄱ') => Some('ㄺ'),
        ('ㅂ', 'ㅅ') => Some('ㅄ'),
        ('ㅅ', 'ㅅ') => Some('ㅆ'),
        ('ㄱ', 'ㅅ') => Some('ㄳ'),
        ('ㄴ', 'ㅎ') => Some('ㄶ'),
        ('ㄹ', 'ㅁ') => Some('ㄻ'),
        ('ㄹ', 'ㅂ') => Some('ㄼ'),
        ('ㄹ', 'ㅅ') => Some('ㄽ'),
        ('ㄹ', 'ㅌ') => Some('ㄾ'),
        ('ㄹ', 'ㅍ') => Some('ㄿ'),
        ('ㄹ', 'ㅎ') => Some('ㅀ'),
        _ => None,
    }
}

pub(crate) fn decompose_composite_final(c: char) -> Option<(char, char)> {
    match c {
        'ㄲ' => Some(('ㄱ', 'ㄱ')),
        'ㄵ' => Some(('ㄴ', 'ㅈ')),
        'ㄺ' => Some(('ㄹ', 'ㄱ')),
        'ㅄ' => Some(('ㅂ', 'ㅅ')),
        'ㅆ' => Some(('ㅅ', 'ㅅ')),
        'ㄳ' => Some(('ㄱ', 'ㅅ')),
        'ㄶ' => Some(('ㄴ', 'ㅎ')),
        'ㄻ' => Some(('ㄹ', 'ㅁ')),
        'ㄼ' => Some(('ㄹ', 'ㅂ')),
        'ㄽ' => Some(('ㄹ', 'ㅅ')),
        'ㄾ' => Some(('ㄹ', 'ㅌ')),
        'ㄿ' => Some(('ㄹ', 'ㅍ')),
        'ㅀ' => Some(('ㄹ', 'ㅎ')),
        _ => None,
    }
}

/// Determines the type of Hangul letter for a given character.
/// Does not work for archaic or non-standard jamo like ᅀ.
/// Classifies a character as Hangul jamo or non-Hangul.
pub fn determine_hangul(c: char) -> Character {
    return if CONSONANTS.contains(c) {
        Character::Hangul(Jamo::Consonant(c))
    } else if VOWELS.contains(c) {
        Character::Hangul(Jamo::Vowel(c))
    } else if COMPOSITE_CONSONANTS.contains(c) {
        Character::Hangul(Jamo::CompositeConsonant(c))
    } else if COMPOSITE_VOWELS.contains(c) {
        Character::Hangul(Jamo::CompositeVowel(c))
    } else {
        Character::NonHangul(c)
    };
}

pub(crate) fn is_valid_double_initial(c: char) -> bool {
    INITIAL_COMPOSITE_CONSONANTS.contains(c)
}

pub(crate) fn is_valid_composite_final(c: char) -> bool {
    FINAL_COMPOSITE_CONSONANTS.contains(c)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Character {
    NonHangul(char),
    Hangul(Jamo),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Jamo {
    Consonant(char),
    CompositeConsonant(char),
    Vowel(char),
    CompositeVowel(char),
}

impl Jamo {
    pub(crate) fn get_char(&self) -> char {
        match self {
            Jamo::Consonant(c)
            | Jamo::CompositeConsonant(c)
            | Jamo::Vowel(c)
            | Jamo::CompositeVowel(c) => *c,
        }
    }
}
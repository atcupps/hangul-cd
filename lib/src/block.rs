use std::fmt::Debug;
use crate::jamo::*;

#[derive(Debug, PartialEq, Eq)]
pub struct HangulBlock {
    pub initial: char,
    pub vowel: char,
    pub final_optional: Option<char>,
}

impl HangulBlock {
    // Extracts the composed Hangul syllable character from the block struct.
    // Assumes all chars are valid jamo.
    pub fn to_char(&self) -> Result<char, u32> {
        // Ensure the initial, vowel, and final are modern Jamo and not
        // compatibility jamo
        let initial = modernize_jamo_initial(self.initial);
        let vowel = modernize_jamo_vowel(self.vowel);
        let final_optional = match self.final_optional {
            Some(c) => Some(modernize_jamo_final(c)),
            None => None,
        };

        // Get u32 representation of chars
        let initial_num = initial as u32;
        let vowel_num = vowel as u32;
        let final_num = match final_optional {
            Some(c) => c as u32,
            None => 0,
        };

        // Calculate indices
        let l_index = initial_num - L_BASE;
        let v_index = vowel_num - V_BASE;
        let t_index = if final_num == 0 {
            0
        } else {
            final_num - T_BASE
        };
        let s_index = (l_index * N_COUNT) + (v_index * T_COUNT) + t_index;

        // Unwrapping because this should only ever be called with valid Hangul
        if let Some(c) = std::char::from_u32(S_BASE + s_index) {
            Ok(c)
        } else {
            Err(S_BASE + s_index)
        }
    }

    pub fn decomposed(
        &self,
    ) -> Result<
        (
            Option<char>,
            Option<char>,
            Option<char>,
            Option<char>,
            Option<char>,
            Option<char>,
        ),
        String,
    > {
        let (i1, i2) = match decompose_composite_initial(self.initial) {
            Some((a, b)) => (Some(a), Some(b)),
            None => (Some(self.initial), None),
        };
        let (v1, v2) = match decompose_composite_vowel(self.vowel) {
            Some((a, b)) => (Some(a), Some(b)),
            None => (Some(self.vowel), None),
        };
        let (f1, f2) = match self.final_optional {
            Some(c) => match decompose_composite_final(c) {
                Some((a, b)) => (Some(a), Some(b)),
                None => (Some(c), None),
            },
            None => (None, None),
        };
        Ok((i1, i2, v1, v2, f1, f2))
    }
}

// Convert compatibility jamo to modern jamo.

pub(crate) fn hangul_blocks_vec_to_string(blocks: &Vec<HangulBlock>) -> Result<String, String> {
    let mut result = String::new();
    for block in blocks {
        match block.to_char() {
            Ok(c) => result.push(c),
            Err(codepoint) => {
                return Err(format!(
                    "Failed to convert HangulBlock: {:?} to char. Invalid codepoint: U+{:X}",
                    block, codepoint
                ));
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_hangul_identifies_valid_consonants() {
        let consonants = "ㅂㅈㄷㄱㅅㅁㄴㅇㄹㅎㅋㅌㅊㅍ";
        for c in consonants.chars() {
            let result = determine_hangul(c);
            assert!(
                result == Character::Hangul(Jamo::Consonant(c)),
                "Failed on consonant: {}; got result: {:?}",
                c,
                result
            );
        }
    }

    #[test]
    fn determine_hangul_identifies_valid_vowels() {
        let vowels = "ㅛㅕㅑㅐㅔㅒㅖㅗㅓㅏㅣㅠㅜㅡ";
        for c in vowels.chars() {
            let result = determine_hangul(c);
            assert!(
                result == Character::Hangul(Jamo::Vowel(c)),
                "Failed on vowel: {}; got result: {:?}",
                c,
                result
            );
        }
    }

    #[test]
    fn determine_hangul_double_initials() {
        let compound_letters = "ㄲㄸㅃㅆㅉ";
        for c in compound_letters.chars() {
            let result = determine_hangul(c);
            assert!(
                result == Character::Hangul(Jamo::CompositeConsonant(c)),
                "Failed on compound letter: {}; got result: {:?}",
                c,
                result
            );
        }
    }

    #[test]
    fn determine_hangul_composite_vowels() {
        let compound_letters = "ㅘㅙㅚㅝㅞㅟㅢ";
        for c in compound_letters.chars() {
            let result = determine_hangul(c);
            assert!(
                result == Character::Hangul(Jamo::CompositeVowel(c)),
                "Failed on compound letter: {}; got result: {:?}",
                c,
                result
            );
        }
    }

    #[test]
    fn determine_hangul_composite_finals() {
        let compound_letters = "ㄲㄳㄵㄶㄺㄻㄼㄽㄾㄿㅀㅄ";
        for c in compound_letters.chars() {
            let result = determine_hangul(c);
            assert!(
                result == Character::Hangul(Jamo::CompositeConsonant(c)),
                "Failed on compound letter: {}; got result: {:?}",
                c,
                result
            );
        }
    }

    #[test]
    fn determine_hangul_non_hangul() {
        let non_hangul_chars = "ABCxyz123!@# ";
        for c in non_hangul_chars.chars() {
            let result = determine_hangul(c);
            assert!(
                result == Character::NonHangul(c),
                "Failed on non-Hangul char: {}; got result: {:?}",
                c,
                result
            );
        }
    }

    #[test]
    fn test_hangul_block_to_char() {
        let block = HangulBlock {
            initial: 'ㄱ',
            vowel: 'ㅏ',
            final_optional: Some('ㄴ'),
        };
        let result = block.to_char();
        assert_eq!(result, Ok('간'));

        let block_no_final = HangulBlock {
            initial: 'ㅂ',
            vowel: 'ㅗ',
            final_optional: None,
        };
        let result_no_final = block_no_final.to_char();
        assert_eq!(result_no_final, Ok('보'));
    }

    #[test]
    fn test_hangul_blocks_vec_to_string() {
        let blocks = vec![
            HangulBlock {
                initial: 'ㅇ',
                vowel: 'ㅏ',
                final_optional: Some('ㄴ'),
            },
            HangulBlock {
                initial: 'ㄴ',
                vowel: 'ㅕ',
                final_optional: Some('ㅇ'),
            },
            HangulBlock {
                initial: 'ㅎ',
                vowel: 'ㅏ',
                final_optional: None,
            },
            HangulBlock {
                initial: 'ㅅ',
                vowel: 'ㅔ',
                final_optional: None,
            },
            HangulBlock {
                initial: 'ㅇ',
                vowel: 'ㅛ',
                final_optional: None,
            },
        ];
        let result = hangul_blocks_vec_to_string(&blocks);
        assert_eq!(result, Ok("안녕하세요".to_string()));
    }
}

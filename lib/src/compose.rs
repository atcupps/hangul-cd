use std::fmt::Debug;

use crate::{block::*, jamo::*};

#[derive(Debug)]
pub struct HangulWordComposer {
    prev_blocks: Vec<HangulBlock>,
    cur_block: BlockComposer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PushResult {
    Success,
    StartNewBlockNoPop,
    PopAndStartNewBlock,
    InvalidHangul,
    NonHangul,
}

#[derive(Debug, PartialEq, Eq)]
enum BlockCompositionState {
    /// nothing, waiting for first consonant
    ExpectingInitial,

    /// ex. ㄷ -> ㄸ or 다
    ExpectingDoubleInitialOrVowel,

    /// ex. ㄸ -> 따
    ExpectingVowel,

    /// ex. 두 -> 둬 or 둔
    ExpectingCompositeVowelOrFinal,

    /// ex. 둬 -> 뒁
    ExpectingFinal,

    /// ex. 달 -> 닳 or 다래
    ExpectingCompositeFinal,

    /// ex. 닳 -> 달하
    ExpectingNextBlock,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BlockComposer {
    state: BlockCompositionState,
    initial_first: Option<char>,
    initial_second: Option<char>,
    vowel_first: Option<char>,
    vowel_second: Option<char>,
    final_first: Option<char>,
    final_second: Option<char>,
}

enum BlockCompletionStatus {
    Complete(HangulBlock),
    Incomplete(char),
}

enum BlockPopStatus {
    PoppedAndShouldContinue(Jamo),
    PoppedAndShouldRemove(Jamo),
    None,
}

impl BlockComposer {
    pub(crate) fn new() -> Self {
        BlockComposer {
            state: BlockCompositionState::ExpectingInitial,
            initial_first: None,
            initial_second: None,
            vowel_first: None,
            vowel_second: None,
            final_first: None,
            final_second: None,
        }
    }

    fn push(&mut self, letter: &Jamo) -> PushResult {
        match self.state {
            BlockCompositionState::ExpectingInitial => self.try_push_initial(letter),
            BlockCompositionState::ExpectingDoubleInitialOrVowel => {
                self.try_push_double_initial_or_vowel(letter)
            }
            BlockCompositionState::ExpectingVowel => self.try_push_vowel(letter),
            BlockCompositionState::ExpectingCompositeVowelOrFinal => {
                self.try_push_composite_vowel_or_final(letter)
            }
            BlockCompositionState::ExpectingFinal => self.try_push_final(letter),
            BlockCompositionState::ExpectingCompositeFinal => self.try_push_composite_final(letter),
            BlockCompositionState::ExpectingNextBlock => self.try_push_next_block(letter),
        }
    }

    fn pop(&mut self) -> BlockPopStatus {
        if let Some(c) = self.final_second.take() {
            self.state = BlockCompositionState::ExpectingCompositeFinal;
            BlockPopStatus::PoppedAndShouldContinue(Jamo::Consonant(c))
        } else if let Some(c) = self.final_first.take() {
            self.state = match self.vowel_second {
                Some(_) => BlockCompositionState::ExpectingFinal,
                None => BlockCompositionState::ExpectingCompositeVowelOrFinal,
            };
            BlockPopStatus::PoppedAndShouldContinue(Jamo::Consonant(c))
        } else if let Some(c) = self.vowel_second.take() {
            self.state = BlockCompositionState::ExpectingCompositeVowelOrFinal;
            BlockPopStatus::PoppedAndShouldContinue(Jamo::Vowel(c))
        } else if let Some(c) = self.vowel_first.take() {
            self.state = match self.initial_second {
                Some(_) => BlockCompositionState::ExpectingVowel,
                None => BlockCompositionState::ExpectingDoubleInitialOrVowel,
            };
            BlockPopStatus::PoppedAndShouldContinue(Jamo::Vowel(c))
        } else if let Some(c) = self.initial_second.take() {
            self.state = BlockCompositionState::ExpectingVowel;
            BlockPopStatus::PoppedAndShouldContinue(Jamo::Consonant(c))
        } else if let Some(c) = self.initial_first.take() {
            self.state = BlockCompositionState::ExpectingInitial;
            BlockPopStatus::PoppedAndShouldRemove(Jamo::Consonant(c))
        } else {
            self.state = BlockCompositionState::ExpectingInitial;
            BlockPopStatus::None
        }
    }

    pub(crate) fn pop_end_consonant(&mut self) -> Option<Jamo> {
        if let Some(c) = self.final_second.take() {
            Some(Jamo::Consonant(c))
        } else if let Some(c) = self.final_first.take() {
            Some(Jamo::Consonant(c))
        } else {
            None
        }
    }

    fn try_push_initial(&mut self, letter: &Jamo) -> PushResult {
        match letter {
            Jamo::Consonant(c) => {
                self.initial_first = Some(*c);
                self.state = BlockCompositionState::ExpectingDoubleInitialOrVowel;
                PushResult::Success
            }
            Jamo::CompositeConsonant(c) => {
                if is_valid_double_initial(*c) {
                    self.initial_first = Some(*c);
                    self.state = BlockCompositionState::ExpectingVowel;
                    PushResult::Success
                } else {
                    PushResult::InvalidHangul
                }
            }
            _ => PushResult::InvalidHangul,
        }
    }

    fn try_push_double_initial_or_vowel(&mut self, letter: &Jamo) -> PushResult {
        match letter {
            Jamo::Consonant(c) => {
                if let Some(initial) = self.initial_first {
                    if create_composite_initial(initial, *c).is_some() {
                        self.initial_second = Some(*c);
                        self.state = BlockCompositionState::ExpectingVowel;
                        PushResult::Success
                    } else {
                        PushResult::InvalidHangul
                    }
                } else {
                    PushResult::InvalidHangul
                }
            }
            Jamo::Vowel(c) => {
                self.vowel_first = Some(*c);
                self.state = BlockCompositionState::ExpectingCompositeVowelOrFinal;
                PushResult::Success
            }
            Jamo::CompositeVowel(c) => {
                if let Some((v1, v2)) = decompose_composite_vowel(*c) {
                    self.vowel_first = Some(v1);
                    self.vowel_second = Some(v2);
                    self.state = BlockCompositionState::ExpectingFinal;
                    PushResult::Success
                } else {
                    PushResult::InvalidHangul
                }
            }
            Jamo::CompositeConsonant(_) => PushResult::InvalidHangul,
        }
    }

    fn try_push_vowel(&mut self, letter: &Jamo) -> PushResult {
        match letter {
            Jamo::Vowel(c) => {
                self.vowel_first = Some(*c);
                self.state = BlockCompositionState::ExpectingCompositeVowelOrFinal;
                PushResult::Success
            }
            Jamo::CompositeVowel(c) => {
                if let Some((v1, v2)) = decompose_composite_vowel(*c) {
                    self.vowel_first = Some(v1);
                    self.vowel_second = Some(v2);
                    self.state = BlockCompositionState::ExpectingFinal;
                    PushResult::Success
                } else {
                    PushResult::InvalidHangul
                }
            }
            _ => PushResult::InvalidHangul,
        }
    }

    fn try_push_composite_vowel_or_final(&mut self, letter: &Jamo) -> PushResult {
        match letter {
            Jamo::Vowel(c) => {
                if let Some(v1) = self.vowel_first {
                    if create_composite_vowel(v1, *c).is_some() {
                        self.initial_second = Some(*c);
                        self.state = BlockCompositionState::ExpectingFinal;
                        PushResult::Success
                    } else {
                        PushResult::InvalidHangul
                    }
                } else {
                    PushResult::InvalidHangul
                }
            }
            Jamo::Consonant(c) => {
                self.final_first = Some(*c);
                self.state = BlockCompositionState::ExpectingCompositeFinal;
                PushResult::Success
            }
            Jamo::CompositeConsonant(c) => {
                if is_valid_composite_final(*c) {
                    self.final_first = Some(*c);
                    self.state = BlockCompositionState::ExpectingNextBlock;
                    PushResult::Success
                } else if is_valid_double_initial(*c) {
                    PushResult::StartNewBlockNoPop
                } else {
                    PushResult::InvalidHangul
                }
            }
            _ => PushResult::InvalidHangul,
        }
    }

    fn try_push_final(&mut self, letter: &Jamo) -> PushResult {
        match letter {
            Jamo::Consonant(c) => {
                self.final_first = Some(*c);
                self.state = BlockCompositionState::ExpectingCompositeFinal;
                PushResult::Success
            }
            Jamo::CompositeConsonant(c) => {
                if is_valid_composite_final(*c) {
                    self.final_first = Some(*c);
                    self.state = BlockCompositionState::ExpectingNextBlock;
                    PushResult::Success
                } else if is_valid_double_initial(*c) {
                    PushResult::StartNewBlockNoPop
                } else {
                    PushResult::InvalidHangul
                }
            }
            _ => PushResult::InvalidHangul,
        }
    }

    fn try_push_composite_final(&mut self, letter: &Jamo) -> PushResult {
        match letter {
            Jamo::Consonant(c) => {
                if let Some(f) = self.final_first {
                    if create_composite_final(f, *c).is_some() {
                        self.final_second = Some(*c);
                        self.state = BlockCompositionState::ExpectingNextBlock;
                        PushResult::Success
                    } else {
                        PushResult::StartNewBlockNoPop
                    }
                } else {
                    PushResult::InvalidHangul
                }
            }
            Jamo::CompositeConsonant(c) => {
                if is_valid_double_initial(*c) {
                    PushResult::StartNewBlockNoPop
                } else {
                    PushResult::InvalidHangul
                }
            }
            Jamo::Vowel(_) | Jamo::CompositeVowel(_) => {
                PushResult::PopAndStartNewBlock
            }
        }
    }

    fn try_push_next_block(&mut self, letter: &Jamo) -> PushResult {
        match letter {
            Jamo::Consonant(_) | Jamo::CompositeConsonant(_) => {
                PushResult::StartNewBlockNoPop
            }
            Jamo::Vowel(_) | Jamo::CompositeVowel(_) => {
                PushResult::PopAndStartNewBlock
            }
        }
    }

    fn try_as_complete_block(&self) -> Result<BlockCompletionStatus, String> {
        let initial_optional = match (self.initial_first, self.initial_second) {
            (Some(i1), Some(i2)) => Some(
                create_composite_initial(i1, i2)
                    .ok_or_else(|| format!("Invalid double initial consonant: {}{}", i1, i2))?,
            ),
            (Some(i1), None) => Some(i1),
            _ => None,
        };
        let vowel_optional = match (self.vowel_first, self.vowel_second) {
            (Some(v1), Some(v2)) => Some(
                create_composite_vowel(v1, v2)
                    .ok_or_else(|| format!("Invalid composite vowel: {}{}", v1, v2))?,
            ),
            (Some(v1), None) => Some(v1),
            _ => None,
        };
        let final_optional = match (self.final_first, self.final_second) {
            (Some(f1), Some(f2)) => Some(
                create_composite_final(f1, f2)
                    .ok_or_else(|| format!("Invalid composite final consonant: {}{}", f1, f2))?,
            ),
            (Some(f1), None) => Some(f1),
            _ => None,
        };

        match (initial_optional, vowel_optional) {
            (Some(initial), Some(vowel)) => Ok(BlockCompletionStatus::Complete(HangulBlock {
                initial,
                vowel,
                final_optional,
            })),
            (Some(initial), None) => Ok(BlockCompletionStatus::Incomplete(initial)),
            (None, Some(vowel)) => Ok(BlockCompletionStatus::Incomplete(vowel)),
            (None, None) => {
                Err("Cannot form block: missing initial consonant and vowel".to_string())
            }
        }
    }

    fn block_as_string(&self) -> Result<Option<char>, String> {
        match self.try_as_complete_block()? {
            BlockCompletionStatus::Complete(block) => block
                .to_char()
                .map(Some)
                .map_err(|e| format!("Error converting block to char: U+{:04X}", e)),
            BlockCompletionStatus::Incomplete(c) => Ok(Some(c)),
        }
    }

    pub(crate) fn from_composed_block(block: &HangulBlock) -> Result<Self, String> {
        let mut result = BlockComposer::new();
        let (i1, i2, v1, v2, f1, f2) = block.decomposed()?;

        if f2.is_some() {
            result.state = BlockCompositionState::ExpectingNextBlock;
        } else if f1.is_some() {
            result.state = BlockCompositionState::ExpectingCompositeFinal;
        } else if v2.is_some() {
            result.state = BlockCompositionState::ExpectingFinal;
        } else if v1.is_some() {
            result.state = BlockCompositionState::ExpectingCompositeVowelOrFinal;
        }
        // Anything after this shouldn't happen. But this won't return an error
        // because it's conceivable that a manually constructed HangulBlock
        // leads to one of these states occuring. This may lead to undefined
        // behavior.
        else if i2.is_some() {
            result.state = BlockCompositionState::ExpectingVowel;
        } else if i1.is_some() {
            result.state = BlockCompositionState::ExpectingDoubleInitialOrVowel;
        } else {
            result.state = BlockCompositionState::ExpectingInitial;
        }

        result.initial_first = i1;
        result.initial_second = i2;
        result.vowel_first = v1;
        result.vowel_second = v2;
        result.final_first = f1;
        result.final_second = f2;

        Ok(result)
    }
}

impl HangulWordComposer {
    pub fn new() -> Self {
        HangulWordComposer {
            prev_blocks: Vec::new(),
            cur_block: BlockComposer::new(),
        }
    }

    pub fn push_char(&mut self, c: char) -> PushResult {
        match determine_hangul(c) {
            Character::Hangul(hl) => self.push(&hl),
            Character::NonHangul(_) => PushResult::NonHangul,
        }
    }

    pub fn push(&mut self, letter: &Jamo) -> PushResult {
        self.cur_block.push(letter)
    }

    pub fn pop(&mut self) -> Result<Option<Jamo>, String> {
        match self.cur_block.pop() {
            BlockPopStatus::PoppedAndShouldContinue(l) => Ok(Some(l)),
            BlockPopStatus::PoppedAndShouldRemove(l) => {
                self.prev_block_to_cur()?;
                Ok(Some(l))
            }
            BlockPopStatus::None => {
                self.prev_block_to_cur()?;
                Ok(None)
            }
        }
    }

    fn prev_block_to_cur(&mut self) -> Result<(), String> {
        if let Some(last_block) = self.prev_blocks.pop() {
            self.cur_block = BlockComposer::from_composed_block(&last_block)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn pop_and_start_new_block(&mut self, letter: Jamo) -> Result<(), String> {
        match self.cur_block.pop_end_consonant() {
            Some(l) => {
                self.complete_current_block()?;
                self.cur_block.push(&l);
                match self.cur_block.push(&letter) {
                    PushResult::Success => Ok(()),
                    _ => Err(format!(
                        "Error starting new block with letter: {:?}",
                        letter
                    )),
                }
            }
            None => Err(format!(
                "Could not pop end consonant in function start_new_block with letter: {:?}",
                letter
            )),
        }
    }

    pub fn start_new_block(&mut self, letter: Jamo) -> Result<(), String> {
        self.complete_current_block()?;
        match self.cur_block.push(&letter) {
            PushResult::Success => Ok(()),
            _ => Err(format!(
                "Error starting new block with letter: {:?}",
                letter
            )),
        }
    }

    pub fn as_string(&self) -> Result<String, String> {
        let mut result = hangul_blocks_vec_to_string(&self.prev_blocks)?;
        let cur_as_char = self.cur_block.block_as_string()?;
        if let Some(c) = cur_as_char {
            result.push(c);
        }
        Ok(result)
    }

    fn complete_current_block(&mut self) -> Result<(), String> {
        match self.cur_block.try_as_complete_block()? {
            BlockCompletionStatus::Complete(block) => {
                self.prev_blocks.push(block);
                self.cur_block = BlockComposer::new();
                Ok(())
            }
            BlockCompletionStatus::Incomplete(c) => Err(format!(
                "Cannot complete current block: incomplete block state, leftover char: {}",
                c
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HangulWordComposerPushLetterTestCase {
        input: Vec<Jamo>,
        expected_final_word_state: PushResult,
        expected_final_block_state: BlockCompositionState,
        expected_prev_blocks: Vec<HangulBlock>,
    }

    fn run_test_cases(cases: Vec<HangulWordComposerPushLetterTestCase>) {
        for case in &cases {
            let mut composer = HangulWordComposer::new();
            let mut final_word_state = PushResult::Success;
            for letter in &case.input {
                final_word_state = composer.push(letter);
            }
            assert_eq!(
                final_word_state, case.expected_final_word_state,
                "Final WORD state did not match expected. Composer: {:?}",
                composer
            );
            assert_eq!(
                composer.cur_block.state, case.expected_final_block_state,
                "Final BLOCK state did not match expected. Composer: {:?}",
                composer
            );
            assert_eq!(
                composer.prev_blocks, case.expected_prev_blocks,
                "Previous blocks did not match expected.",
            );
        }
    }

    #[test]
    fn single_block_composition_valid() {
        let test_cases: Vec<HangulWordComposerPushLetterTestCase> = vec![
            HangulWordComposerPushLetterTestCase {
                input: vec![Jamo::Consonant('ㄱ')],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingDoubleInitialOrVowel,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![Jamo::Consonant('ㄱ'), Jamo::Consonant('ㄱ')],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingVowel,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㄱ'),
                    Jamo::Consonant('ㄱ'),
                    Jamo::Vowel('ㅜ'),
                ],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingCompositeVowelOrFinal,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㄱ'),
                    Jamo::Consonant('ㄱ'),
                    Jamo::Vowel('ㅜ'),
                    Jamo::Vowel('ㅓ'),
                ],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingFinal,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㄱ'),
                    Jamo::Consonant('ㄱ'),
                    Jamo::Vowel('ㅜ'),
                    Jamo::Vowel('ㅓ'),
                    Jamo::Consonant('ㄹ'),
                ],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingCompositeFinal,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㄱ'),
                    Jamo::Consonant('ㄱ'),
                    Jamo::Vowel('ㅜ'),
                    Jamo::Vowel('ㅓ'),
                    Jamo::Consonant('ㄹ'),
                    Jamo::Consonant('ㅎ'),
                ],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingNextBlock,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㄱ'),
                    Jamo::Consonant('ㄱ'),
                    Jamo::Vowel('ㅜ'),
                    Jamo::Vowel('ㅓ'),
                    Jamo::Consonant('ㄹ'),
                    Jamo::Consonant('ㅎ'),
                    Jamo::Vowel('ㅏ'),
                ],
                expected_final_word_state: PushResult::PopAndStartNewBlock,
                expected_final_block_state: BlockCompositionState::ExpectingNextBlock,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::CompositeConsonant('ㅃ'),
                    Jamo::Vowel('ㅣ'),
                    Jamo::CompositeConsonant('ㄳ'),
                ],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingNextBlock,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㅈ'),
                    Jamo::CompositeVowel('ㅚ'),
                ],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingFinal,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::CompositeConsonant('ㅉ'),
                    Jamo::CompositeVowel('ㅢ'),
                    Jamo::CompositeConsonant('ㅃ'),
                ],
                expected_final_word_state: PushResult::StartNewBlockNoPop,
                expected_final_block_state: BlockCompositionState::ExpectingFinal,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㅇ'),
                    Jamo::Vowel('ㅣ'),
                    Jamo::Consonant('ㅅ'),
                    Jamo::Consonant('ㅅ'),
                ],
                expected_final_word_state: PushResult::Success,
                expected_final_block_state: BlockCompositionState::ExpectingNextBlock,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㅇ'),
                    Jamo::Vowel('ㅣ'),
                    Jamo::Consonant('ㅅ'),
                    Jamo::Consonant('ㅅ'),
                    Jamo::Consonant('ㅅ'),
                ],
                expected_final_word_state: PushResult::StartNewBlockNoPop,
                expected_final_block_state: BlockCompositionState::ExpectingNextBlock,
                expected_prev_blocks: vec![],
            },
        ];

        run_test_cases(test_cases);
    }

    #[test]
    fn single_block_composition_invalid() {
        let test_cases: Vec<HangulWordComposerPushLetterTestCase> = vec![
            HangulWordComposerPushLetterTestCase {
                input: vec![Jamo::Consonant('ㄱ'), Jamo::Consonant('ㄹ')],
                expected_final_word_state: PushResult::InvalidHangul,
                expected_final_block_state: BlockCompositionState::ExpectingDoubleInitialOrVowel,
                expected_prev_blocks: vec![],
            },
            HangulWordComposerPushLetterTestCase {
                input: vec![
                    Jamo::Consonant('ㄱ'),
                    Jamo::Vowel('ㅏ'),
                    Jamo::Vowel('ㅏ'),
                ],
                expected_final_word_state: PushResult::InvalidHangul,
                expected_final_block_state: BlockCompositionState::ExpectingCompositeVowelOrFinal,
                expected_prev_blocks: vec![],
            },
        ];
        run_test_cases(test_cases);
    }

    #[test]
    fn start_new_block_valid() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(
            composer.push(&Jamo::Consonant('ㄱ')),
            PushResult::Success
        );
        assert_eq!(
            composer.push(&Jamo::Vowel('ㅏ')),
            PushResult::Success
        );
        assert_eq!(
            composer.push(&Jamo::Consonant('ㄴ')),
            PushResult::Success,
        );
        assert_eq!(
            composer.push(&Jamo::Consonant('ㅇ')),
            PushResult::StartNewBlockNoPop,
        );
        assert_eq!(
            composer.start_new_block(Jamo::Consonant('ㅇ')),
            Ok(())
        );
        assert_eq!(
            composer.prev_blocks,
            vec![HangulBlock {
                initial: 'ㄱ',
                vowel: 'ㅏ',
                final_optional: Some('ㄴ'),
            }]
        );
        assert_eq!(
            composer.cur_block.state,
            BlockCompositionState::ExpectingDoubleInitialOrVowel
        );
        assert_eq!(
            composer.push(&Jamo::Vowel('ㅛ')),
            PushResult::Success
        );
        assert_eq!(
            composer.push(&Jamo::CompositeConsonant('ㅉ')),
            PushResult::StartNewBlockNoPop,
        );
        assert_eq!(
            composer.start_new_block(Jamo::CompositeConsonant('ㅉ')),
            Ok(())
        );
        assert_eq!(
            composer.prev_blocks,
            vec![
                HangulBlock {
                    initial: 'ㄱ',
                    vowel: 'ㅏ',
                    final_optional: Some('ㄴ'),
                },
                HangulBlock {
                    initial: 'ㅇ',
                    vowel: 'ㅛ',
                    final_optional: None,
                }
            ]
        );
    }

    #[test]
    fn start_new_block_invalid() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(
            composer.start_new_block(Jamo::Vowel('ㅏ')),
            Err("Cannot form block: missing initial consonant and vowel".to_string())
        );
        let _ = composer.push(&Jamo::Consonant('ㄱ'));
        assert_eq!(
            composer.start_new_block(Jamo::CompositeVowel('ㅘ')),
            Err(
                "Cannot complete current block: incomplete block state, leftover char: ㄱ"
                    .to_string()
            )
        );
    }

    #[test]
    fn push_char_valid() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(composer.push_char('ㄱ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::Success,);
    }

    #[test]
    fn push_char_invalid_hangul() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(composer.push_char('ㄱ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄹ'), PushResult::Success,);
        assert_eq!(composer.push_char('ㄽ'), PushResult::InvalidHangul,);
    }

    #[test]
    fn push_char_next_block() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(composer.push_char('ㄱ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::Success,);
        assert_eq!(composer.push_char('ㅇ'), PushResult::StartNewBlockNoPop,);
    }

    #[test]
    fn push_char_non_hangul() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(composer.push_char('ㄱ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('A'), PushResult::NonHangul,);
    }

    #[test]
    fn test_single_word_안녕하세요_as_string() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::StartNewBlockNoPop);
        assert_eq!(
            composer.start_new_block(Jamo::Consonant('ㄴ')),
            Ok(())
        );
        assert_eq!(composer.push_char('ㅕ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅎ'), PushResult::StartNewBlockNoPop);
        assert_eq!(
            composer.start_new_block(Jamo::Consonant('ㅎ')),
            Ok(())
        );
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅅ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅔ'), PushResult::PopAndStartNewBlock);
        assert_eq!(
            composer.pop_and_start_new_block(Jamo::Vowel('ㅔ')),
            Ok(())
        );
        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅛ'), PushResult::PopAndStartNewBlock);
        assert_eq!(
            composer.pop_and_start_new_block(Jamo::Vowel('ㅛ')),
            Ok(())
        );

        let result_string = composer.as_string().unwrap();
        assert_eq!(result_string, "안녕하세요".to_string());
    }

    #[test]
    fn test_single_word_앖어요_as_string() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅓ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅂ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅅ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅇ'), PushResult::StartNewBlockNoPop);
        assert_eq!(
            composer.start_new_block(Jamo::Consonant('ㅇ')),
            Ok(())
        );
        assert_eq!(composer.push_char('ㅓ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅛ'), PushResult::PopAndStartNewBlock);
        assert_eq!(
            composer.pop_and_start_new_block(Jamo::Vowel('ㅛ')),
            Ok(())
        );

        let result_string = composer.as_string().unwrap();
        assert_eq!(result_string, "없어요".to_string());
    }

    #[test]
    fn test_incomplete_block_as_string() {
        let mut composer = HangulWordComposer::new();

        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);

        let result_string = composer.as_string().unwrap();
        assert_eq!(result_string, "ㅇ".to_string());
    }

    #[test]
    fn test_deletions() {
        let mut composer = HangulWordComposer::new();
        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::StartNewBlockNoPop);
        assert_eq!(
            composer.start_new_block(Jamo::Consonant('ㄴ')),
            Ok(())
        );
        assert_eq!(composer.push_char('ㅕ'), PushResult::Success);

        assert_eq!(composer.pop(), Ok(Some(Jamo::Vowel('ㅕ'))));
        assert_eq!(composer.pop(), Ok(Some(Jamo::Consonant('ㄴ'))));
        assert_eq!(composer.pop(), Ok(Some(Jamo::Consonant('ㄴ'))));
        assert_eq!(composer.pop(), Ok(Some(Jamo::Vowel('ㅏ'))));
        assert_eq!(composer.pop(), Ok(Some(Jamo::Consonant('ㅇ'))));
        assert_eq!(composer.pop(), Ok(None));
    }

    #[test]
    fn test_deletion_then_write_again() {
        let mut composer = HangulWordComposer::new();
        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::Success);

        assert_eq!(composer.pop(), Ok(Some(Jamo::Consonant('ㄴ'))));
        assert_eq!(composer.pop(), Ok(Some(Jamo::Vowel('ㅏ'))));
        assert_eq!(composer.pop(), Ok(Some(Jamo::Consonant('ㅇ'))));

        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::Success);

        let result_string = composer.as_string().unwrap();
        assert_eq!(result_string, "안".to_string());
    }

    #[test]
    fn deletion_removes_empty_block() {
        let mut composer = HangulWordComposer::new();
        assert_eq!(composer.push_char('ㅇ'), PushResult::Success);
        assert_eq!(composer.push_char('ㅏ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::Success);
        assert_eq!(composer.push_char('ㄴ'), PushResult::StartNewBlockNoPop);
        assert_eq!(
            composer.start_new_block(Jamo::Consonant('ㄴ')),
            Ok(())
        );

        assert_eq!(composer.pop(), Ok(Some(Jamo::Consonant('ㄴ'))));
        // if current block is still empty, as_string should fail
        assert_eq!(composer.as_string().unwrap(), "안".to_string());
    }
}

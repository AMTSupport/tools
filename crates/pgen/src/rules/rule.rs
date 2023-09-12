/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::processor::processor::Processor;
use crate::processor::word::Word;
use crate::rules::action::Action;

pub trait Rule {
    type Passable: Default = Option<()>;

    fn process(&self, processor: &mut Processor) {
        let mut passable = self.create_passable();
        self.process_with_passable(processor, &mut passable);
    }

    /// Process the rule.
    fn process_with_passable(&self, processor: &mut Processor, args: &mut Self::Passable) {
        let length = processor.words.len();
        processor.words.iter_mut().fold(None, |previous, current| {
            let last = current.index == (length - 1);
            self.process_word(previous, current, last, args).into_iter().for_each(|action| {
                current.actions.insert(action);
            });

            Some(current)
        });
    }

    /// Apply the rule to the given word.
    ///
    /// This will not actually modify the word directly but instead will
    /// return a [Action] which will be applied to the [Word] later.
    ///
    /// # Arguments
    ///
    /// * `previous` - A reference to the previous [Word] that was processed or none `current` is the first index.
    /// * `current` - A reference to the current [Word] being processed.
    /// * `last` - A boolean indicating if this is the last [Word] in the processor.
    /// * `passable` - A mutable reference to the [Rule::Args] for this rule, this is passed to all following invocation of this rule within the same processor.
    fn process_word(
        &self,
        previous: Option<&Word>,
        current: &Word,
        last: bool,
        passable: &mut Self::Passable,
    ) -> Vec<Action>;

    /// Create a new instance of the object that is passed to all invocations of this rule.
    ///
    /// This is called once per processor.
    /// This will be mutated and passed to all invocations of this rule within the same processor.
    fn create_passable(&self) -> Self::Passable {
        Self::Passable::default()
    }
}

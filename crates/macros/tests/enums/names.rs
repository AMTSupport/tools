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

use macros::EnumNames;
use std::assert_matches::assert_matches;

#[derive(Debug, EnumNames)]
enum Test {
    Apple,
    Banana,
    Cherry,
}

#[test]
fn get_name() {
    assert_eq!(Test::Apple.name(), "Apple");
    assert_eq!(Test::Banana.name(), "Banana");
    assert_eq!(Test::Cherry.name(), "Cherry");
}

#[test]
fn try_from() {
    assert_matches!(Test::try_from("Apple"), Ok(Test::Apple));
    assert_matches!(Test::try_from("Banana"), Ok(Test::Banana));
    assert_matches!(Test::try_from("Cherry"), Ok(Test::Cherry));
}

#[test]
fn try_from_different_case() {
    assert_matches!(Test::try_from("aPpLe"), Ok(Test::Apple));
    assert_matches!(Test::try_from("banana"), Ok(Test::Banana));
    assert_matches!(Test::try_from("CHERRY"), Ok(Test::Cherry));
}

#[test]
fn try_from_unknown() {
    assert_matches!(Test::try_from("Orange"), Err(_));
}

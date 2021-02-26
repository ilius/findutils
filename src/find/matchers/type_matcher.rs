// Copyright 2017 Google Inc.
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use std::error::Error;
use std::fs::FileType;
use std::os::unix::fs::FileTypeExt;
use walkdir::DirEntry;

use super::{Matcher, MatcherIO};

/// This matcher checks the type of the file.
pub struct TypeMatcher {
    file_type_fn: fn(&FileType) -> bool,
}

impl TypeMatcher {
    pub fn new(type_string: &str) -> Result<TypeMatcher, Box<dyn Error>> {
    	#[cfg(unix)]
        let function = match type_string {
            "f" => FileType::is_file,
            "d" => FileType::is_dir,
            "l" => TypeMatcher::is_symlink,
            "b" => FileType::is_block_device,
            "c" => FileType::is_char_device,
            "p" => FileType::is_fifo, // named pipe (FIFO)
            "s" => FileType::is_socket,
            // D: door (Solaris)
            "D" => {
                return Err(From::from(format!(
                    "Type argument {} not supported yet",
                    type_string
                )))
            }
            _ => {
                return Err(From::from(format!(
                    "Unrecognised type argument {}",
                    type_string
                )))
            }
        };
        #[cfg(not(unix))]
        let function = match type_string {
            "f" => FileType::is_file,
            "d" => FileType::is_dir,
            "l" => TypeMatcher::is_symlink,
            _ => {
                return Err(From::from(format!(
                    "Unrecognised type argument {}",
                    type_string
                )))
            }
        };
        Ok(TypeMatcher {
            file_type_fn: function,
        })
    }

    pub fn is_symlink(file_type: &FileType) -> bool {
        // to check -H -L -P flags (not currently supported) here
        // from "man find":
        // l: symbolic link; this is never true if the -L option or the -follow
        // option is in effect, unless the symbolic link is broken.
        // If you want to search for symbolic links when -L is in effect, use -xtype.
        return file_type.is_symlink();
    }

    pub fn new_box(type_string: &str) -> Result<Box<dyn Matcher>, Box<dyn Error>> {
        Ok(Box::new(TypeMatcher::new(type_string)?))
    }
}

impl Matcher for TypeMatcher {
    fn matches(&self, file_info: &DirEntry, _: &mut MatcherIO) -> bool {
        (self.file_type_fn)(&file_info.file_type())
    }
}
#[cfg(test)]

mod tests {
    use super::*;
    use crate::find::matchers::tests::get_dir_entry_for;
    use crate::find::matchers::Matcher;
    use crate::find::tests::FakeDependencies;

    #[test]
    fn file_type_matcher() {
        let file = get_dir_entry_for("test_data/simple", "abbbc");
        let dir = get_dir_entry_for("test_data", "simple");
        let deps = FakeDependencies::new();

        let matcher = TypeMatcher::new(&"f".to_string()).unwrap();
        assert!(!matcher.matches(&dir, &mut deps.new_matcher_io()));
        assert!(matcher.matches(&file, &mut deps.new_matcher_io()));
    }

    #[test]
    fn dir_type_matcher() {
        let file = get_dir_entry_for("test_data/simple", "abbbc");
        let dir = get_dir_entry_for("test_data", "simple");
        let deps = FakeDependencies::new();

        let matcher = TypeMatcher::new(&"d".to_string()).unwrap();
        assert!(matcher.matches(&dir, &mut deps.new_matcher_io()));
        assert!(!matcher.matches(&file, &mut deps.new_matcher_io()));
    }

    #[test]
    fn cant_create_with_invalid_pattern() {
        let result = TypeMatcher::new(&"xxx".to_string());
        assert!(result.is_err());
    }
}

use std::{error::Error, fmt, str::Chars};

use crate::platform::Key;

use crate::{
    client::ClientHandle,
    serialization::{DeserializeError, Deserializer, Serialize, Serializer},
};

#[derive(Debug)]
pub enum KeyParseError {
    UnexpectedEnd,
    InvalidCharacter(char),
}
impl fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "could not finish parsing key"),
            Self::InvalidCharacter(c) => write!(f, "invalid character {}", c),
        }
    }
}
impl Error for KeyParseError {}

#[derive(Debug)]
pub struct KeyParseAllError {
    pub index: usize,
    pub error: KeyParseError,
}
impl fmt::Display for KeyParseAllError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.error.fmt(f)?;
        f.write_fmt(format_args!(" at index: {}", self.index))?;
        Ok(())
    }
}
impl Error for KeyParseAllError {}

pub struct KeyParser<'a> {
    len: usize,
    chars: Chars<'a>,
}
impl<'a> Iterator for KeyParser<'a> {
    type Item = Result<Key, KeyParseAllError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chars.as_str().is_empty() {
            return None;
        }
        match parse_key(&mut self.chars) {
            Ok(key) => Some(Ok(key)),
            Err(error) => Some(Err(KeyParseAllError {
                index: self.len - self.chars.as_str().len(),
                error,
            })),
        }
    }
}

pub fn parse_all_keys<'a>(raw: &'a str) -> KeyParser<'a> {
    KeyParser {
        len: raw.len(),
        chars: raw.chars(),
    }
}

pub fn parse_key(chars: &mut impl Iterator<Item = char>) -> Result<Key, KeyParseError> {
    macro_rules! next {
        () => {
            match chars.next() {
                Some(element) => element,
                None => return Err(KeyParseError::UnexpectedEnd),
            }
        };
    }

    macro_rules! consume {
        ($character:expr) => {
            let c = next!();
            if c != $character {
                return Err(KeyParseError::InvalidCharacter(c));
            }
        };
    }

    macro_rules! consume_str {
        ($str:expr) => {
            for c in $str.chars() {
                consume!(c);
            }
        };
    }

    let key = match next!() {
        '<' => match next!() {
            'b' => {
                consume_str!("ackspace>");
                Key::Backspace
            }
            's' => {
                consume_str!("pace>");
                Key::Char(' ')
            }
            'e' => match next!() {
                'n' => match next!() {
                    't' => {
                        consume_str!("er>");
                        Key::Enter
                    }
                    'd' => {
                        consume!('>');
                        Key::End
                    }
                    c => return Err(KeyParseError::InvalidCharacter(c)),
                },
                's' => {
                    consume_str!("c>");
                    Key::Esc
                }
                c => return Err(KeyParseError::InvalidCharacter(c)),
            },
            'l' => {
                consume!('e');
                match next!() {
                    's' => {
                        consume_str!("s>");
                        Key::Char('<')
                    }
                    'f' => {
                        consume_str!("t>");
                        Key::Left
                    }
                    c => return Err(KeyParseError::InvalidCharacter(c)),
                }
            }
            'g' => {
                consume_str!("reater>");
                Key::Char('>')
            }
            'r' => {
                consume_str!("ight>");
                Key::Right
            }
            'u' => {
                consume_str!("p>");
                Key::Up
            }
            'd' => match next!() {
                'o' => {
                    consume_str!("wn>");
                    Key::Down
                }
                'e' => {
                    consume_str!("lete>");
                    Key::Delete
                }
                c => return Err(KeyParseError::InvalidCharacter(c)),
            },
            'h' => {
                consume_str!("ome>");
                Key::Home
            }
            'p' => {
                consume_str!("age");
                match next!() {
                    'u' => {
                        consume_str!("p>");
                        Key::PageUp
                    }
                    'd' => {
                        consume_str!("own>");
                        Key::PageDown
                    }
                    c => return Err(KeyParseError::InvalidCharacter(c)),
                }
            }
            't' => {
                consume_str!("ab>");
                Key::Tab
            }
            'f' => {
                let c = next!();
                match c.to_digit(10) {
                    Some(d0) => {
                        let c = next!();
                        match c.to_digit(10) {
                            Some(d1) => {
                                consume!('>');
                                let n = d0 * 10 + d1;
                                Key::F(n as _)
                            }
                            None => {
                                if c == '>' {
                                    Key::F(d0 as _)
                                } else {
                                    return Err(KeyParseError::InvalidCharacter(c));
                                }
                            }
                        }
                    }
                    None => return Err(KeyParseError::InvalidCharacter(c)),
                }
            }
            'c' => {
                consume!('-');
                let c = next!();
                let key = if c.is_ascii_alphanumeric() {
                    Key::Ctrl(c)
                } else {
                    return Err(KeyParseError::InvalidCharacter(c));
                };
                consume!('>');
                key
            }
            'a' => {
                consume!('-');
                let c = next!();
                let key = if c.is_ascii_alphanumeric() {
                    Key::Alt(c)
                } else {
                    return Err(KeyParseError::InvalidCharacter(c));
                };
                consume!('>');
                key
            }
            c => return Err(KeyParseError::InvalidCharacter(c)),
        },
        c @ '>' => return Err(KeyParseError::InvalidCharacter(c)),
        c => {
            if c.is_ascii() {
                Key::Char(c)
            } else {
                return Err(KeyParseError::InvalidCharacter(c));
            }
        }
    };

    Ok(key)
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Key::None => Ok(()),
            Key::Backspace => f.write_str("<backspace>"),
            Key::Enter => f.write_str("<enter>"),
            Key::Left => f.write_str("<left>"),
            Key::Right => f.write_str("<right>"),
            Key::Up => f.write_str("<up>"),
            Key::Down => f.write_str("<down>"),
            Key::Home => f.write_str("<home>"),
            Key::End => f.write_str("<end>"),
            Key::PageUp => f.write_str("<pageup>"),
            Key::PageDown => f.write_str("<pagedown>"),
            Key::Tab => f.write_str("<tab>"),
            Key::Delete => f.write_str("<delete>"),
            Key::F(n) => f.write_fmt(format_args!("<f{}>", n)),
            Key::Char(' ') => f.write_str("<space>"),
            Key::Char('<') => f.write_str("<less>"),
            Key::Char('>') => f.write_str("<greater>"),
            Key::Char(c) => f.write_fmt(format_args!("{}", c)),
            Key::Ctrl(c) => f.write_fmt(format_args!("<c-{}>", c)),
            Key::Alt(c) => f.write_fmt(format_args!("<a-{}>", c)),
            Key::Esc => f.write_str("<esc>"),
        }
    }
}

fn serialize_key<S>(key: Key, serializer: &mut S)
where
    S: Serializer,
{
    match key {
        Key::None => 0u8.serialize(serializer),
        Key::Backspace => 1u8.serialize(serializer),
        Key::Enter => 2u8.serialize(serializer),
        Key::Left => 3u8.serialize(serializer),
        Key::Right => 4u8.serialize(serializer),
        Key::Up => 5u8.serialize(serializer),
        Key::Down => 6u8.serialize(serializer),
        Key::Home => 7u8.serialize(serializer),
        Key::End => 8u8.serialize(serializer),
        Key::PageUp => 9u8.serialize(serializer),
        Key::PageDown => 10u8.serialize(serializer),
        Key::Tab => 11u8.serialize(serializer),
        Key::Delete => 12u8.serialize(serializer),
        Key::F(n) => {
            13u8.serialize(serializer);
            n.serialize(serializer);
        }
        Key::Char(c) => {
            14u8.serialize(serializer);
            c.serialize(serializer);
        }
        Key::Ctrl(c) => {
            15u8.serialize(serializer);
            c.serialize(serializer);
        }
        Key::Alt(c) => {
            16u8.serialize(serializer);
            c.serialize(serializer);
        }
        Key::Esc => 17u8.serialize(serializer),
    }
}

fn deserialize_key<'de, D>(deserializer: &mut D) -> Result<Key, DeserializeError>
where
    D: Deserializer<'de>,
{
    let discriminant = u8::deserialize(deserializer)?;
    match discriminant {
        0 => Ok(Key::None),
        1 => Ok(Key::Backspace),
        2 => Ok(Key::Enter),
        3 => Ok(Key::Left),
        4 => Ok(Key::Right),
        5 => Ok(Key::Up),
        6 => Ok(Key::Down),
        7 => Ok(Key::Home),
        8 => Ok(Key::End),
        9 => Ok(Key::PageUp),
        10 => Ok(Key::PageDown),
        11 => Ok(Key::Tab),
        12 => Ok(Key::Delete),
        13 => {
            let n = Serialize::deserialize(deserializer)?;
            Ok(Key::F(n))
        }
        14 => {
            let c = Serialize::deserialize(deserializer)?;
            Ok(Key::Char(c))
        }
        15 => {
            let c = Serialize::deserialize(deserializer)?;
            Ok(Key::Ctrl(c))
        }
        16 => {
            let c = Serialize::deserialize(deserializer)?;
            Ok(Key::Alt(c))
        }
        17 => Ok(Key::Esc),
        _ => Err(DeserializeError),
    }
}

// TODO: change from Option<TargetClient> to TargetClient where TargetClient the enum
// enum TargetClient {
// FromConnection,
// Focused,
// Handle(ClientHandle),
// }
pub enum ClientEvent<'a> {
    Key(Option<ClientHandle>, Key),
    Resize(Option<ClientHandle>, u16, u16),
    Command(Option<ClientHandle>, &'a str),
}

impl<'de> Serialize<'de> for ClientEvent<'de> {
    fn serialize<S>(&self, serializer: &mut S)
    where
        S: Serializer,
    {
        match self {
            ClientEvent::Key(handle, key) => {
                0u8.serialize(serializer);
                handle.serialize(serializer);
                serialize_key(*key, serializer);
            }
            ClientEvent::Resize(handle, width, height) => {
                1u8.serialize(serializer);
                handle.serialize(serializer);
                width.serialize(serializer);
                height.serialize(serializer);
            }
            ClientEvent::Command(handle, command) => {
                2u8.serialize(serializer);
                handle.serialize(serializer);
                command.serialize(serializer);
            }
        }
    }

    fn deserialize<D>(deserializer: &mut D) -> Result<Self, DeserializeError>
    where
        D: Deserializer<'de>,
    {
        let discriminant = u8::deserialize(deserializer)?;
        match discriminant {
            0 => {
                let handle = Serialize::deserialize(deserializer)?;
                let key = deserialize_key(deserializer)?;
                Ok(ClientEvent::Key(handle, key))
            }
            1 => {
                let handle = Serialize::deserialize(deserializer)?;
                let width = u16::deserialize(deserializer)?;
                let height = u16::deserialize(deserializer)?;
                Ok(ClientEvent::Resize(handle, width, height))
            }
            2 => {
                let handle = Serialize::deserialize(deserializer)?;
                let command = <&str>::deserialize(deserializer)?;
                Ok(ClientEvent::Command(handle, command))
            }
            _ => Err(DeserializeError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_parsing() {
        assert_eq!(
            Key::Backspace,
            parse_key(&mut "<backspace>".chars()).unwrap()
        );
        assert_eq!(Key::Char(' '), parse_key(&mut "<space>".chars()).unwrap());
        assert_eq!(Key::Enter, parse_key(&mut "<enter>".chars()).unwrap());
        assert_eq!(Key::Left, parse_key(&mut "<left>".chars()).unwrap());
        assert_eq!(Key::Right, parse_key(&mut "<right>".chars()).unwrap());
        assert_eq!(Key::Up, parse_key(&mut "<up>".chars()).unwrap());
        assert_eq!(Key::Down, parse_key(&mut "<down>".chars()).unwrap());
        assert_eq!(Key::Home, parse_key(&mut "<home>".chars()).unwrap());
        assert_eq!(Key::End, parse_key(&mut "<end>".chars()).unwrap());
        assert_eq!(Key::PageUp, parse_key(&mut "<pageup>".chars()).unwrap());
        assert_eq!(Key::PageDown, parse_key(&mut "<pagedown>".chars()).unwrap());
        assert_eq!(Key::Tab, parse_key(&mut "<tab>".chars()).unwrap());
        assert_eq!(Key::Delete, parse_key(&mut "<delete>".chars()).unwrap());
        assert_eq!(Key::Esc, parse_key(&mut "<esc>".chars()).unwrap());

        for n in 1..=99 {
            let s = format!("<f{}>", n);
            assert_eq!(Key::F(n as _), parse_key(&mut s.chars()).unwrap());
        }

        assert_eq!(Key::Ctrl('z'), parse_key(&mut "<c-z>".chars()).unwrap());
        assert_eq!(Key::Ctrl('0'), parse_key(&mut "<c-0>".chars()).unwrap());
        assert_eq!(Key::Ctrl('9'), parse_key(&mut "<c-9>".chars()).unwrap());

        assert_eq!(Key::Alt('a'), parse_key(&mut "<a-a>".chars()).unwrap());
        assert_eq!(Key::Alt('z'), parse_key(&mut "<a-z>".chars()).unwrap());
        assert_eq!(Key::Alt('0'), parse_key(&mut "<a-0>".chars()).unwrap());
        assert_eq!(Key::Alt('9'), parse_key(&mut "<a-9>".chars()).unwrap());

        assert_eq!(Key::Char('a'), parse_key(&mut "a".chars()).unwrap());
        assert_eq!(Key::Char('z'), parse_key(&mut "z".chars()).unwrap());
        assert_eq!(Key::Char('0'), parse_key(&mut "0".chars()).unwrap());
        assert_eq!(Key::Char('9'), parse_key(&mut "9".chars()).unwrap());
        assert_eq!(Key::Char('_'), parse_key(&mut "_".chars()).unwrap());
        assert_eq!(Key::Char('<'), parse_key(&mut "<less>".chars()).unwrap());
        assert_eq!(Key::Char('>'), parse_key(&mut "<greater>".chars()).unwrap());
        assert_eq!(Key::Char('\\'), parse_key(&mut "\\".chars()).unwrap());
    }

    #[test]
    fn key_serialization() {
        use crate::serialization::{DeserializationSlice, SerializationBuf};

        macro_rules! assert_key_serialization {
            ($key:expr) => {
                let mut buf = SerializationBuf::default();
                let _ = serialize_key($key, &mut buf);
                let slice = buf.as_slice();
                let mut deserializer = DeserializationSlice::from_slice(slice);
                assert!(!deserializer.as_slice().is_empty());
                match deserialize_key(&mut deserializer) {
                    Ok(key) => assert_eq!($key, key),
                    Err(_) => assert!(false),
                }
            };
        }

        assert_key_serialization!(Key::None);
        assert_key_serialization!(Key::Backspace);
        assert_key_serialization!(Key::Enter);
        assert_key_serialization!(Key::Left);
        assert_key_serialization!(Key::Right);
        assert_key_serialization!(Key::Up);
        assert_key_serialization!(Key::Down);
        assert_key_serialization!(Key::Home);
        assert_key_serialization!(Key::End);
        assert_key_serialization!(Key::PageUp);
        assert_key_serialization!(Key::PageDown);
        assert_key_serialization!(Key::Tab);
        assert_key_serialization!(Key::Delete);
        assert_key_serialization!(Key::F(0));
        assert_key_serialization!(Key::F(9));
        assert_key_serialization!(Key::F(12));
        assert_key_serialization!(Key::F(99));
        assert_key_serialization!(Key::Char('a'));
        assert_key_serialization!(Key::Char('z'));
        assert_key_serialization!(Key::Char('A'));
        assert_key_serialization!(Key::Char('Z'));
        assert_key_serialization!(Key::Char('0'));
        assert_key_serialization!(Key::Char('9'));
        assert_key_serialization!(Key::Char('$'));
        assert_key_serialization!(Key::Ctrl('a'));
        assert_key_serialization!(Key::Ctrl('z'));
        assert_key_serialization!(Key::Ctrl('A'));
        assert_key_serialization!(Key::Ctrl('Z'));
        assert_key_serialization!(Key::Ctrl('0'));
        assert_key_serialization!(Key::Ctrl('9'));
        assert_key_serialization!(Key::Ctrl('$'));
        assert_key_serialization!(Key::Alt('a'));
        assert_key_serialization!(Key::Alt('z'));
        assert_key_serialization!(Key::Alt('A'));
        assert_key_serialization!(Key::Alt('Z'));
        assert_key_serialization!(Key::Alt('0'));
        assert_key_serialization!(Key::Alt('9'));
        assert_key_serialization!(Key::Alt('$'));
        assert_key_serialization!(Key::Esc);
    }
}

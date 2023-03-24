use indexmap::IndexMap;
use std::collections::HashSet;

pub type Map = IndexMap<String, Vec<String>>;
pub type List = Vec<String>;

pub struct ArgMap {
    pub boolean: HashSet<String>,
}

impl Default for ArgMap {
    fn default() -> Self {
        Self::new()
    }
}

impl ArgMap {
    /// Create a new ArgMap instance.
    pub fn new() -> Self {
        Self {
            boolean: HashSet::new(),
        }
    }
    /// Set a key to be treated as a boolean argument, where an argument that follows a boolean
    /// argument will not be treated as the key's value.
    pub fn boolean<T>(mut self, key: T) -> Self
    where
        T: ToString,
    {
        self.boolean.insert(key.to_string());
        self
    }
    /// Set multiple keys to be treated as boolean arguments, where an argument that follows a boolean
    /// argument will not be treated as the key's value.
    pub fn booleans<T>(mut self, keys: &[T]) -> Self
    where
        T: ToString,
    {
        for key in keys.iter() {
            self.boolean.insert(key.to_string());
        }
        self
    }
    /// Parse an iterator of string arguments into a 2-tuple of positional arguments and a
    /// HashMap mapping String keys to Vec<String> values.
    pub fn parse<T>(&mut self, input: impl Iterator<Item = T>) -> (List, Map)
    where
        T: ToString,
    {
        let mut args: List = vec![];
        let mut argv: Map = IndexMap::new();
        let mut key: Option<String> = None;
        let mut dashdash = false;
        for x in input {
            let s = x.to_string();
            if dashdash {
                args.push(s);
                continue;
            }
            if s == "--" {
                dashdash = true;
            } else if s == "-" {
                args.push(s);
            } else if let Some(k) = s.strip_prefix("--") {
                if let Some(k) = &key {
                    argv.insert(k.clone(), vec![]);
                    key = None;
                }
                let k = k.to_string();
                if let Some(i) = k.find('=') {
                    set(&mut argv, &k[0..i].to_string(), &k[i + 1..]);
                } else if self.boolean.contains(&k) {
                    set_bool(&mut argv, &k)
                } else {
                    key = Some(k);
                }
            } else if s.starts_with('-') {
                if let Some(k) = &key {
                    if is_num(&s[1..2]) {
                        set(&mut argv, k, &s.to_string());
                        key = None;
                        continue;
                    }
                    set_bool(&mut argv, k);
                    argv.insert(k.clone(), vec![]);
                    key = None;
                }
                if let Some(i) = s.find('=') {
                    let sk = s[1..i].to_string();
                    let sv = s[i + 1..].to_string();
                    set(&mut argv, &sk, &sv);
                } else {
                    let mut jump = false;
                    for i in 1..s.len() - 1 {
                        let k = s[i..i + 1].to_string();
                        if let Some(sk) = &key {
                            if is_num(&k) || short_break(&k) {
                                set(&mut argv, sk, &s[i..]);
                                key = None;
                                jump = true;
                                break;
                            } else {
                                set_bool(&mut argv, sk);
                            }
                            key = None;
                        }
                        if self.boolean.contains(&k) {
                            set_bool(&mut argv, &k);
                        } else {
                            key = Some(k);
                        }
                    }
                    if jump {
                        continue;
                    }
                    let k = s[s.len() - 1..].to_string();
                    if let Some(sk) = &key {
                        if self.boolean.contains(&k) {
                            set_bool(&mut argv, sk);
                            set_bool(&mut argv, &k);
                        } else if is_num(&k) || short_break(&k) {
                            set(&mut argv, sk, &k);
                            key = None;
                        } else {
                            set_bool(&mut argv, sk);
                            key = Some(k);
                        }
                    } else if self.boolean.contains(&k) {
                        set_bool(&mut argv, &k);
                    } else {
                        key = Some(k);
                    }
                }
            } else if let Some(k) = key {
                set(&mut argv, &k, &s);
                key = None;
            } else {
                args.push(s);
            }
        }
        if let Some(k) = key {
            set_bool(&mut argv, &k);
        }
        (args, argv)
    }
}

/// Parse an iterator of string arguments into a 2-tuple of positional arguments and a
/// HashMap mapping String keys to Vec<String> values.
pub fn parse<T>(input: impl Iterator<Item = T>) -> (List, Map)
where
    T: ToString,
{
    ArgMap::default().parse(input)
}

fn is_num(s: &str) -> bool {
    s.chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
}
fn short_break(s: &str) -> bool {
    s.chars()
        .next()
        .map(|c| !c.is_alphabetic())
        .unwrap_or(false)
}

fn set(argv: &mut Map, key: &String, value: &str) {
    if let Some(values) = argv.get_mut(key) {
        values.push(value.to_string());
    } else {
        argv.insert(key.clone(), vec![value.to_string()]);
    }
}
fn set_bool(argv: &mut Map, key: &String) {
    if !argv.contains_key(key) {
        argv.insert(key.clone(), vec![]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_junk0() {
        let (args, argv) = parse(
            [
                "--long",
                "5",
                "-x",
                "6",
                "-n3",
                "hello",
                "-xvf",
                "whatever.tgz",
                "-y=cool",
                "-x7",
                "world",
                "--z=13",
                "-z",
                "12",
                "--",
                "hmm",
            ]
            .iter(),
        );
        assert_eq![args, vec!["hello", "world", "hmm"]];
        assert_eq![
            argv,
            hash(
                [
                    ("long", vec!["5"]),
                    ("x", vec!["6", "7"]),
                    ("n", vec!["3"]),
                    ("v", vec![]),
                    ("f", vec!["whatever.tgz"]),
                    ("y", vec!["cool"]),
                    ("z", vec!["13", "12"]),
                ]
                .iter()
            )
        ];
    }

    #[test]
    fn parse_junk1() {
        let (args, argv) = parse(
            [
                "--hey=what",
                "-x",
                "5",
                "-x",
                "6",
                "hi",
                "-zn9",
                "-j",
                "3",
                "-i",
                "q",
                "-5",
                "--n",
                "-1312",
                "-xvf",
                "payload.tgz",
                "-j=zzz",
                "-",
                "whatever",
                "-w3",
                "--",
                "-cool",
                "--yes=xyz",
            ]
            .iter(),
        );
        assert_eq![args, vec!["hi", "-", "whatever", "-cool", "--yes=xyz"]];
        assert_eq![
            argv,
            hash(
                [
                    ("hey", vec!["what"]),
                    ("x", vec!["5", "6"]),
                    ("z", vec![]),
                    ("j", vec!["3", "zzz"]),
                    ("i", vec!["q"]),
                    ("5", vec![]),
                    ("n", vec!["9", "-1312"]),
                    ("v", vec![]),
                    ("f", vec!["payload.tgz"]),
                    ("w", vec!["3"]),
                ]
                .iter()
            )
        ];
    }

    #[test]
    fn parse_empty() {
        let empty: Vec<String> = vec![];
        let (args, argv) = parse(empty.iter());
        assert_eq![args, empty];
        assert_eq![argv, hash([].iter())];
    }

    #[test]
    fn parse_one_long_bool() {
        let empty: Vec<String> = vec![];
        let (args, argv) = parse(["--one"].iter());
        assert_eq![args, empty];
        assert_eq![argv, hash([("one", vec![]),].iter())];
    }

    #[test]
    fn parse_one_short_bool() {
        let empty: Vec<String> = vec![];
        let (args, argv) = parse(["-z"].iter());
        assert_eq![args, empty];
        assert_eq![argv, hash([("z", vec![]),].iter())];
    }

    #[test]
    fn parse_bool_at_dashdash() {
        let empty: Vec<String> = vec![];
        let (args, argv) = parse(["--q", "--"].iter());
        assert_eq![args, empty];
        assert_eq![argv, hash([("q", vec![]),].iter())];
    }

    #[test]
    fn parse_negative_number_value() {
        let empty: Vec<String> = vec![];
        let (args, argv) = parse(["--n", "-555"].iter());
        assert_eq![args, empty];
        assert_eq![argv, hash([("n", vec!["-555"]),].iter())];
    }

    #[test]
    fn parse_cluster_number() {
        let empty: Vec<String> = vec![];
        let (args, argv) = parse(["-abcdef123456"].iter());
        assert_eq![args, empty];
        assert_eq![
            argv,
            hash(
                [
                    ("a", vec![]),
                    ("b", vec![]),
                    ("c", vec![]),
                    ("d", vec![]),
                    ("e", vec![]),
                    ("f", vec!["123456"]),
                ]
                .iter()
            )
        ];
    }

    #[test]
    fn parse_single_boolean() {
        let (args, argv) = ArgMap::default()
            .boolean("q")
            .parse(["-x", "5", "-q", "1234", "--z=789"].iter());
        assert_eq![args, vec!["1234"]];
        assert_eq![
            argv,
            hash([("x", vec!["5"]), ("q", vec![]), ("z", vec!["789"]),].iter())
        ];
    }

    #[test]
    fn parse_boolean_nonalpha_break() {
        let empty: Vec<String> = vec![];
        let (args, argv) = ArgMap::default()
            .boolean("q")
            .parse(["-w-5", "-qrs@4"].iter());
        assert_eq![args, empty];
        assert_eq![
            argv,
            hash(
                [
                    ("w", vec!["-5"]),
                    ("q", vec![]),
                    ("r", vec![]),
                    ("s", vec!["@4"]),
                ]
                .iter()
            )
        ];
    }

    #[test]
    fn parse_booleans_slice() {
        let (args, argv) = ArgMap::default()
            .booleans(&["q", "z"])
            .parse(["-q", "x", "-z", "y"].iter());
        assert_eq![args, vec!["x", "y"]];
        assert_eq![argv, hash([("q", vec![]), ("z", vec![]),].iter())];
    }

    #[test]
    fn parse_boolean_vec_ref() {
        let (args, argv) = ArgMap::default()
            .booleans(&["q", "z"])
            .parse(["-q", "x", "-z", "y"].iter());
        assert_eq![args, vec!["x", "y"]];
        assert_eq![argv, hash([("q", vec![]), ("z", vec![]),].iter())];
    }

    fn hash<'a>(
        i: impl Iterator<Item = &'a (&'a str, Vec<&'a str>)>,
    ) -> IndexMap<String, Vec<String>> {
        i.map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }
}

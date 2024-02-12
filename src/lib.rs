use std::error::Error;

pub trait RegexParse: Sized {
    fn parse(text: &str) -> Result<Self, Box<dyn Error>>;
}

macro_rules! impl_regex {
    ($($ty:ty)+) => {
        $(
            impl RegexParse for $ty {
                fn parse(text: &str) -> Result<Self, Box<dyn Error>> {
                    Ok(text.trim().parse::<$ty>()?)
                }
            }
        )+
    }
}

impl_regex! { i8 i16 i32 i64 isize u8 u16 u32 u64 usize }
impl_regex! { f32 f64 bool String }

impl<T: RegexParse> RegexParse for Vec<T> {
    fn parse(text: &str) -> Result<Self, Box<dyn Error>> {
        let mut result = vec![];
        for item in text.split(',') {
            result.push(T::parse(item)?);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use regex::Regex;

    use crate::RegexParse;

    use regex_parse_proc_macro::regex;

    fn test(string: &str) -> Result<Vec<i32>, Box<dyn Error>> {
        let mut result = vec![];

        for item in string.split('.') {
            result.push(i32::parse(item)?);
        }

        Ok(result)
    }

    #[derive(Default, Debug, PartialEq)]
    #[regex(regex = r"Game(?<id>[^:]+):(?<winning_nums>[^;]+);(?<nums>.*)")]
    pub struct Game {
        pub id: i32,
        #[regex(method = "test")]
        pub winning_nums: Vec<i32>,
        pub nums: Vec<i32>,
    }

    #[test]
    pub fn test_parse_game() {
        let string = "Game 1: 1. 2. 3. 4; 1, 2, 3".to_string();

        let game = Game::parse(&string);

        assert_eq!(
            game.unwrap(),
            Game {
                id: 1,
                winning_nums: vec![1, 2, 3, 4],
                nums: vec![1, 2, 3]
            }
        )
    }
}

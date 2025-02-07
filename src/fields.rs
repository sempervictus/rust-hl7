use super::separators::Separators;
use super::*;
use std::fmt::Display;
use std::ops::Index;

/// Represents a single field inside the HL7.  Note that fields can include repeats, components and sub-components.
/// See [the spec](http://www.hl7.eu/HL7v2x/v251/std251/ch02.html#Heading13) for more info
#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub source: &'a str,
    pub delims: Separators,
    pub repeats: Vec<&'a str>,
    pub components: Vec<Vec<&'a str>>,
    pub subcomponents: Vec<Vec<Vec<&'a str>>>,
}

impl<'a> Field<'a> {
    /// Convert the given line of text into a field.
    pub fn parse<S: Into<&'a str>>(
        input: S,
        delims: &Separators,
    ) -> Result<Field<'a>, Hl7ParseError> {
        let input = input.into();
        let repeats: Vec<&'a str> = input.split(delims.repeat).collect();
        let components: Vec<Vec<&'a str>> = repeats
            .iter()
            .map(|r| r.split(delims.component).collect::<Vec<&'a str>>())
            .collect();
        let subcomponents: Vec<Vec<Vec<&'a str>>> = components
            .iter()
            .map(|r| {
                r.iter()
                    .map(|c| c.split(delims.subcomponent).collect::<Vec<&'a str>>())
                    .collect::<Vec<Vec<&'a str>>>()
            })
            .collect();
        let field = Field {
            source: input,
            delims: *delims,
            repeats,
            components,
            subcomponents,
        };
        Ok(field)
    }

    /// Used to hide the removal of NoneError for #2...  If passed `Some()` value it returns a field with that value.  If passed `None() it returns an `Err(Hl7ParseError::MissingRequiredValue{})`
    pub fn parse_mandatory(
        input: Option<&'a str>,
        delims: &Separators,
    ) -> Result<Field<'a>, Hl7ParseError> {
        match input {
            Some(string_value) => Field::parse(string_value, delims),
            None => Err(Hl7ParseError::MissingRequiredValue {}),
        }
    }

    /// Converts a possibly blank string into a possibly blank field!  
    /// Note this handles optional fields, not the nul (`""`) value.
    pub fn parse_optional(
        input: Option<&'a str>,
        delims: &Separators,
    ) -> Result<Option<Field<'a>>, Hl7ParseError> {
        match input {
            None => Ok(None),
            Some(x) if x.is_empty() => Ok(None),
            Some(x) => Ok(Some(Field::parse(x, delims)?)),
        }
    }

    /// Compatibility method to get the underlying value of this field.
    #[inline]
    pub fn value(&self) -> &'a str {
        self.source
    }

    /// Export value to str
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.source
    }

    /// Access string reference of a Field component by String index
    /// Adjust the index by one as medical people do not count from zero
    pub fn query<'b, S>(&self, sidx: S) -> &'a str
    where
        S: Into<&'b str>,
    {
        let sidx = sidx.into();
        let parts = sidx.split('.').collect::<Vec<&str>>();

        if parts.len() == 1 {
            let stringnums = parts[0]
                .chars()
                .filter(|c| c.is_digit(10))
                .collect::<String>();
            let idx: usize = stringnums.parse().unwrap();

            self[idx - 1]
        } else if parts.len() == 2 {
            let stringnums = parts[0]
                .chars()
                .filter(|c| c.is_digit(10))
                .collect::<String>();

            let idx0: usize = stringnums.parse().unwrap();

            let stringnums = parts[1]
                .chars()
                .filter(|c| c.is_digit(10))
                .collect::<String>();

            let idx1: usize = stringnums.parse().unwrap();

            self[(idx0 - 1, idx1 - 1)]
        } else {
            ""
        }
    }
}

impl<'a> Display for Field<'a> {
    /// Required for to_string() and other formatter consumers
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl<'a> Clone for Field<'a> {
    /// Creates a new Message object using a clone of the original's source
    fn clone(&self) -> Self {
        Field::parse(self.source, &self.delims.clone()).unwrap()
    }
}

impl<'a> Index<usize> for Field<'a> {
    type Output = &'a str;
    /// Access string reference of a Field component by numeric index
    fn index(&self, idx: usize) -> &Self::Output {
        if idx > self.repeats.len() - 1 {
            return &""; //TODO: We're returning &&str here which doesn't seem right?!?
        }

        &self.repeats[idx]
    }
}

impl<'a> Index<(usize, usize)> for Field<'a> {
    type Output = &'a str;
    /// Access string reference of a Field subcomponent by numeric index
    fn index(&self, idx: (usize, usize)) -> &Self::Output {
        if idx.0 > self.repeats.len() - 1 || idx.1 > self.components[idx.0].len() - 1 {
            return &""; //TODO: We're returning &&str here which doesn't seem right?!?
        }

        &self.components[idx.0][idx.1]
    }
}

impl<'a> Index<(usize, usize, usize)> for Field<'a> {
    type Output = &'a str;
    /// Access string reference of a Field subcomponent by numeric index
    fn index(&self, idx: (usize, usize, usize)) -> &Self::Output {
        if idx.0 > self.repeats.len() - 1
            || idx.1 > self.components[idx.0].len() - 1
            || idx.2 > self.subcomponents[idx.0][idx.1].len() - 1
        {
            return &""; //TODO: We're returning &&str here which doesn't seem right?!?
        }

        &self.subcomponents[idx.0][idx.1][idx.2]
    }
}

#[cfg(feature = "string_index")]
impl<'a> Index<String> for Field<'a> {
    type Output = &'a str;

    /// Access string reference of a Field component by String index
    #[cfg(feature = "string_index")]
    fn index(&self, sidx: String) -> &Self::Output {
        let parts = sidx.split('.').collect::<Vec<&str>>();
        match parts.len() {
            1 => {
                let stringnums = parts[0]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>();
                let idx: usize = stringnums.parse().unwrap();

                &self[idx - 1]
            }
            2 => {
                let stringnums = parts[0]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>();

                let idx0: usize = stringnums.parse().unwrap();

                let stringnums = parts[1]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>();

                let idx1: usize = stringnums.parse().unwrap();

                &self[(idx0 - 1, idx1 - 1)]
            }
            3 => {
                let stringnums = parts[0]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>();

                let idx0: usize = stringnums.parse().unwrap();

                let stringnums = parts[1]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>();

                let idx1: usize = stringnums.parse().unwrap();

                let stringnums = parts[2]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>();

                let idx2: usize = stringnums.parse().unwrap();

                &self[(idx0 - 1, idx1 - 1, idx2 - 1)]
            }
            _ => &"",
        }
    }
}

#[cfg(feature = "string_index")]
impl<'a> Index<&str> for Field<'a> {
    type Output = &'a str;

    /// Access Segment, Field, or sub-field string references by string index
    #[cfg(feature = "string_index")]
    fn index(&self, idx: &str) -> &Self::Output {
        &self[String::from(idx)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conditional_parse_handles_none() {
        let d = Separators::default();

        //if we pass a none value, we get a None back
        match Field::parse_optional(None, &d) {
            Ok(None) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_conditional_parse_handles_empty_string() {
        let d = Separators::default();

        //an empty string (as seen when `split()`ing) should be none
        match Field::parse_optional(Some(""), &d) {
            Ok(None) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_conditional_parse_handles_value_string() {
        let d = Separators::default();

        //an empty string (as seen when `split()`ing) should be none
        match Field::parse_optional(Some("xxx"), &d) {
            Ok(Some(field)) => assert_eq!(field.value(), "xxx"),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_mandatory_handles_some_value() {
        let d = Separators::default();

        match Field::parse_mandatory(Some("xxx"), &d) {
            Ok(field) => assert_eq!(field.value(), "xxx"),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_mandatory_throws_on_none() {
        let d = Separators::default();

        match Field::parse_mandatory(None, &d) {
            Err(Hl7ParseError::MissingRequiredValue()) => assert!(true),
            _ => assert!(false),
        }
    }
    #[test]
    fn test_parse_repeats() {
        let d = Separators::default();
        let f = Field::parse_mandatory(Some("x&x^y&y~a&a^b&b"), &d).unwrap();
        assert_eq!(f.repeats.len(), 2)
    }

    #[test]
    fn test_parse_components() {
        let d = Separators::default();
        let f = Field::parse_mandatory(Some("xxx^yyy"), &d).unwrap();
        assert_eq!(f.components[0].len(), 2)
    }

    #[test]
    fn test_parse_subcomponents() {
        let d = Separators::default();
        let f = Field::parse_mandatory(Some("xxx^yyy&zzz"), &d).unwrap();
        assert_eq!(f.subcomponents[0][1].len(), 2)
    }

    #[test]
    fn test_to_string() {
        let d = Separators::default();
        let f = Field::parse_mandatory(Some("xxx^yyy&zzz"), &d).unwrap();
        assert_eq!(f.to_string(), String::from("xxx^yyy&zzz"))
    }

    #[test]
    fn test_clone() {
        let d = Separators::default();
        let f = Field::parse_mandatory(Some("xxx^yyy&zzz"), &d).unwrap();
        assert_eq!(f.to_string(), f.clone().as_str())
    }

    #[test]
    fn test_uint_index() {
        let d = Separators::default();
        let f = Field::parse_mandatory(Some("xxx^yyy&zzz"), &d).unwrap();
        assert_eq!(f[(0, 1)], "yyy&zzz");
        assert_eq!(f[(0, 1, 1)], "zzz");
    }

    #[test]
    fn test_string_query() {
        let d = Separators::default();
        let f = Field::parse_mandatory(Some("x&x^y&y~a&a^b&b"), &d).unwrap();
        let idx0 = String::from("R2");
        let oob = "R2.C3";
        assert_eq!(f.query(&*idx0), "a&a^b&b");
        assert_eq!(f.query("R2.C2"), "b&b");
        assert_eq!(f.query(oob), "");
    }

    #[cfg(feature = "string_index")]
    mod string_index_tests {
        use super::*;
        #[test]
        fn test_string_index() {
            let d = Separators::default();
            let f = Field::parse_mandatory(Some("x&x^y&y~a&a^b&b"), &d).unwrap();
            assert_eq!(f["R2"], "a&a^b&b");
            assert_eq!(f["R2.C2"], "b&b");
            assert_eq!(f["R2.C3"], "");
        }
    }
}

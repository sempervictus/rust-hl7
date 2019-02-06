//! This module parses a whole pipe-delimited style HL7 V2 message by reading forwards through the string only.  It is expected that only a single message is passed at a time.
//! Note that this parses to constituent values, but makes no effort to intepret those values (ie no strong-typing of segments etc)
//! or to interpret the values (coercian to numeric values etc).  Utility API's [are being added](Field::get_as_string) to better handle these fields
pub struct ForwardsMessageParser {}

use super::*;

/// A repeat of a field is a set of 0 or more sub component values.
/// Currently all values are stored as their original string representations.  Methods to convert
/// the values to their HL7-spec types is outside the scope of the parser.
#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
pub struct Repeat2<'a> {
    pub components: Vec<&'a str>, //reference to orignal input string
}

/// A Field is a single 'value between the pipes'.
/// It consists of (0 or more) repeats.
#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
pub struct Field2<'a> {
    pub repeats: Vec<Repeat2<'a>>,
}

/// A single segment, 0x13 delimited line from a source HL7 message consisting of multiple fields.
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Segment2<'a> {
    pub fields: Vec<Field2<'a>>,
}

/// A Message is an entire HL7 message parsed into it's consitituent segments, fields, repeats and subcomponents
/// It consists of (1 or more) Segments.
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Message2<'a> {
    pub segments: Vec<Segment2<'a>>,
}

impl ForwardsMessageParser {
    /// Parses an entire HL7 message into it's component values
    pub fn parse_message<'a>(&mut self, input: &'a String) -> Message2<'a> {
        let mut result = Message2 {
            segments: Vec::new(),
        };

        let delims = Seperators::default();
        let mut repeat_start_index: usize = 0;
        let mut repeats: Vec<Repeat2> = Vec::new();
        let mut fields: Vec<Field2> = Vec::new();

        for (i, c) in input.chars().enumerate() {
            //if our input string is not 1-byte/char, we're in bit trouble here...
            if c == delims.repeat {
                repeat_start_index =
                    self.finish_repeat(repeat_start_index, i, &input, &mut repeats);
                assert_eq!(repeat_start_index, 0);
            } else if c == delims.field {
                // first, finish off any in-flight repeat values
                repeat_start_index =
                    self.finish_repeat(repeat_start_index, i, &input, &mut repeats);

                let f = self.finish_field(&mut repeats);
                fields.push(f);
            } else if c == delims.segment {
                repeat_start_index =
                    self.finish_repeat(repeat_start_index, i, &input, &mut repeats);
                let f = self.finish_field(&mut repeats);
                fields.push(f);

                let seg = ForwardsMessageParser::finish_segment(&mut fields);
                result.segments.push(seg);
            }

            //else just roll on to the next char
        }

        result
    }

    /// Termninates the in-flight repeat value at the current index.
    fn finish_repeat<'a>(
        &mut self,
        start_index: usize,
        end_index: usize,
        input: &'a String,
        repeats: &mut Vec<Repeat2<'a>>,
    ) -> usize {
        let repeat_value = &input[start_index..end_index]; //get a slice ref to the source string

        let r = Repeat2 {
            components: vec![repeat_value], //TODO: Componenet handling
        };

        repeats.push(r);
        end_index + 1
    }

    fn finish_field<'a>(&mut self, repeats: &mut Vec<Repeat2<'a>>) -> Field2<'a> {
        let f = Field2 {
            repeats: repeats.to_owned(), // Get an owned copy of the Vec we've built up in self...
        };

        repeats.clear(); // start gathering repeats again from scratch.  don't shrink so we can re-use the allocated mem next time through.

        f
    }

    fn finish_segment<'a>(fields: &mut Vec<Field2<'a>>) -> Segment2<'a> {
        Segment2 {
            fields: fields.to_owned(),
        } // take ownership and store it in the segment
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let pid = "PID|||555-44-4444||EVERYWOMAN^EVE^E^^^^L|JONES|19620320|F|||153 FERNWOOD DR.^^STATESVILLE^OH^35292||(206)3345232|(206)752-121||||AC555444444||67-A4335^OH^20030520\r".to_string();
        let mut parser = ForwardsMessageParser {};
        let m = parser.parse_message(&pid);

        assert_eq!(m.segments.len(), 1);
        assert_eq!(m.segments[0].fields.len(), 21);
        assert_eq!(
            m.segments[0].fields[3].repeats[0].components[0],
            "555-44-4444"
        );

        println!("{:?}", m);
    }

}

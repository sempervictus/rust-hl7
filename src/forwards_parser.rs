//! This module parses a whole pipe-delimited style HL7 V2 message by reading forwards through the string only.  It is expected that only a single message is passed at a time.
//! Note that this parses to constituent values, but makes no effort to intepret those values (ie no strong-typing of segments etc)
//! or to interpret the values (coercian to numeric values etc).  Utility API's [are being added](Field2::get_all_as_string) to better handle these fields
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
pub struct Message2 {
    pub input: String,
}

impl<'a> Repeat2<'a> {
    /// Returns all components for this repeat as a single string.  If multiple components are present they are joined
    /// with the standard HL7 '^' separator.
    pub fn get_as_string(&'a self) -> String {
        if self.components.len() == 0 {
            return "".to_string();
        } else {
            self.components.join("^") //TODO: How to convert char to &str in a sane way so we can use the Seperators?
        }
    }
}

impl<'a> Field2<'a> {
    pub fn get_all_as_string(&'a self) -> String {
        if self.repeats.len() == 0 {
            return "".to_string();
        }

        self.repeats.iter().map(|r| r.get_as_string()).join("~")
    }
}

impl Message2 {
    pub fn get_field(&self, segment_type: &str, field_index: usize) -> String {
        let delims = Seperators::default();

        let segment = self
            .input
            .split(delims.segment)
            .filter(|line| line[..3] == *segment_type)
            .next();

        match segment {
            None => "".to_string(), // TODO: Throw error no segment found
            Some(line) => line
                .split(delims.field)
                .nth(field_index)
                .unwrap()
                .to_string(),
        }

        /*let matching_segments = self.get_segments(segment_type);
        let segment = matching_segments[0];
        let result = segment.fields[field_index].get_all_as_string();
        result*/
    }
}

impl ForwardsMessageParser {
    /// Parses an entire HL7 message into it's component values
    pub fn parse_message(&mut self, input: &String) -> Message2 {
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
        let s = Segment2 {
            fields: fields.to_owned(),
        }; // take ownership and store it in the segment

        fields.clear();

        s
    }
}

//! This module parses a whole pipe-delimited style HL7 V2 message by reading forwards through the string only.  It is expected that only a single message is passed at a time.
//! Note that this parses to constituent values, but makes no effort to intepret those values (ie no strong-typing of segments etc)
//! or to interpret the values (coercian to numeric values etc).  Utility API's [are being added](Field::get_as_string) to better handle these fields
pub struct ForwardsMessageParser {
    // index of the start of the current repeat.
    repeat_start: usize,
    repeats: Vec<Repeat>, //no allocs until pushed... the set of repeats for the current field
    fields: Vec<Field>,   //all the fields we've found so far.  Cleared at the end of the segment.
    delims: Seperators,
}

use super::*;

impl ForwardsMessageParser {
    pub fn new() -> ForwardsMessageParser {
        ForwardsMessageParser {
            repeat_start: 0,
            repeats: Vec::new(),
            fields: Vec::new(),
            delims: Seperators::default(), //TODO: Custom delimiters
        }
    }

    /// Parses an entire HL7 message into it's component values
    pub fn parse_message(&mut self, input: String) -> Message {
        let mut result = Message::empty();

        for (i, c) in input.chars().enumerate() {
            //if our input string is not 1-byte/char, we're in bit trouble here...

            if c == self.delims.repeat {
                self.finish_repeat(i, &input);
            } else if c == self.delims.field {
                // first, finish off any in-flight repeat values
                self.finish_repeat(i, &input);
                self.finish_field();
            } else if c == self.delims.segment {
                self.finish_repeat(i, &input);
                self.finish_field();

                let seg = self.finish_segment();
                result.segments.push(seg);
            }
        }

        result
    }

    /// Termninates the in-flight repeat value at the current index.
    fn finish_repeat(&mut self, current: usize, input: &String) {
        let repeat_value = &input[self.repeat_start..current];

        let r = Repeat {
            components: vec![repeat_value.to_string()], //TODO: Componenet handling
        };

        self.repeats.push(r); //This to_string() is unfortunate, but required for our FFI work where the caller is reponsible for the string alloc, and slices can't be gauranteed to remnain accurate...
        self.repeat_start = current + 1;
    }

    fn finish_field(&mut self) {
        let f = Field {
            repeats: self.repeats.to_vec(), // Get an owned copy of the Vec we've built up in self...
        };

        self.fields.push(f);

        self.repeats.clear(); // start gathering repeats again from scratch.  don't shrink so we can re-use the allocated mem next time through.
    }

    fn finish_segment(&mut self) -> Segment {
        let s = Segment {
            fields: self.fields.to_vec(),
        };

        self.fields.clear();

        s
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let pid = "PID|||555-44-4444||EVERYWOMAN^EVE^E^^^^L|JONES|19620320|F|||153 FERNWOOD DR.^^STATESVILLE^OH^35292||(206)3345232|(206)752-121||||AC555444444||67-A4335^OH^20030520\r".to_string();
        let mut parser = ForwardsMessageParser::new();
        let m = parser.parse_message(pid);

        assert_eq!(m.segments.len(), 1);
        assert_eq!(m.segments[0].fields.len(), 21);
    }

}

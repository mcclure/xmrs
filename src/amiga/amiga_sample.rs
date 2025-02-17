use super::serde_helper::deserialize_string_22;
use bincode::ErrorKind;
use serde::Deserialize;
use std::fmt::{Debug, Formatter};

use crate::prelude::*;

#[derive(Default, Deserialize)]
pub struct AmigaSample {
    #[serde(deserialize_with = "deserialize_string_22")]
    pub name: String,
    pub length: u16,
    pub finetune: u8,
    pub volume: u8,
    pub repeat_offset: u16,
    pub repeat_length: u16,
}

impl Debug for AmigaSample {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sample: {} (v:{}, f:{}, l:{}, ro:{}, rl:{})\n",
            self.name,
            self.volume,
            self.finetune,
            self.length,
            self.repeat_offset,
            self.repeat_length
        )
    }
}

impl AmigaSample {
    pub fn load(ser_sample: &[u8]) -> Result<(&[u8], Self), Box<ErrorKind>> {
        match bincode::deserialize::<AmigaSample>(&ser_sample) {
            Ok(mut aspl) => {
                // bincode::DefaultOptions::new().with_big_endian() seems not working?
                // manual ROR with * 2...
                aspl.length = 2 * aspl.length.rotate_right(8);
                aspl.repeat_offset = 2 * aspl.repeat_offset.rotate_right(8);
                aspl.repeat_length = 2 * aspl.repeat_length.rotate_right(8);
                Ok((&ser_sample[30..], aspl))
            }
            Err(e) => Err(e),
        }
    }

    pub fn to_sample(&self) -> Sample {
        let f = ((self.finetune << 4) as i8) as f32 / 127.0;
        let ro = if self.repeat_offset < self.length {
            self.repeat_offset
        } else {
            0
        };
        let rl = if ro + self.repeat_length <= self.length {
            self.repeat_length
        } else {
            0
        };
        let flag = if rl > 2 {
            LoopType::Forward
        } else {
            LoopType::No
        };

        Sample {
            name: self.name.clone(),
            loop_start: ro as u32,
            loop_length: rl as u32,
            volume: self.volume as f32 / 64.0,
            finetune: f,
            flags: flag,
            panning: 0.5,
            relative_note: 0,
            data: crate::prelude::SampleDataType::Depth8(vec![]),
        }
    }
}

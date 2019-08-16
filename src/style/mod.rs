use std::string::ToString;

pub enum Orientation {
  Vertical,
  Horizontal,
}


// TODO intern
impl ToString for Orientation {
    fn to_string(&self) -> String {
      match self {
        Orientation::Vertical => "Vertical".into(),
        Orientation::Horizontal => "Horizontal".into(),
      }
    }
}

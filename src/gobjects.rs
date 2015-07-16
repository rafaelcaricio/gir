use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::str::FromStr;
use toml::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GStatus {
    Manual,     //already generated
    Generate,
    Comment,
    Ignore,
}

impl Default for GStatus {
    fn default() -> GStatus { GStatus::Ignore }
}

impl FromStr for GStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manual" => Ok(GStatus::Manual),
            "generate" => Ok(GStatus::Generate),
            "comment" => Ok(GStatus::Comment),
            "ignore" => Ok(GStatus::Ignore),
            _ => Err("Wrong object status".into())
        }
    }
}

/// Info about GObject descendant
#[derive(Clone, Debug)]
pub struct GObject {
    pub name: String,
    pub status: GStatus,
}

impl Default for GObject {
    fn default() -> GObject {
        GObject {
            name: "Default".into(),
            status: Default::default(),
        }
    }
}

//TODO: ?change to HashMap<String, GStatus>
pub type GObjects =  HashMap<String, GObject>;

pub fn parse_toml(toml_objects: &Value) -> GObjects {
    let mut objects = GObjects::new();
    for toml_object in toml_objects.as_slice().unwrap() {
        let gobject = parse_object(toml_object);
        objects.insert(gobject.name.clone(), gobject);
    }
    objects
}

fn parse_object(toml_object: &Value) -> GObject {
    let name = toml_object.lookup("name").expect("Object name not defined")
        .as_str().unwrap().into();

    let status = match toml_object.lookup("status") {
        Some(value) => GStatus::from_str(value.as_str().unwrap()).unwrap_or(Default::default()),
        None => Default::default(),
    };
    GObject { name: name, status: status }
}

pub fn parse_status_shorthands(objects: &mut GObjects, toml: &Value) {
    use self::GStatus::*;
    for &status in [Ignore].iter() {
        parse_status_shorthand(objects, status, toml);
    }
}

fn parse_status_shorthand(objects: &mut GObjects, status: GStatus, toml: &Value) {
    let name = format!("options.{:?}", status).to_ascii_lowercase();
    match toml.lookup(&name) {
        Some(&Value::Array(ref a)) =>
            for name_ in a.iter().map(|s| s.as_str().unwrap()) {
                match objects.get(name_) {
                    None => {
                        objects.insert(name_.into(), GObject {
                            name: name_.into(),
                            status: GStatus::Ignore,
                        });
                    },
                    Some(_) => panic!("Bad name in {}: {} already defined", name, name_),
                }
            },
        _ => (),
    }
}

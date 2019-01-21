pub type Entries = im::ordmap::OrdMap<String, Entry>;

pub trait EntriesStuff {
    // Deserializes JSON.
    fn deserialize(json: &str) -> Self;
    // Loads and deserializes a file. Assumes the file exists because it will
    // be checked as part of the CLI application.
    fn load(filename: &std::path::PathBuf) -> Self;

    // Serializes into JSON.
    fn serialize(&self) -> String;
    // Serializes and writes a file.
    fn save(&self, filename: &std::path::PathBuf) -> std::io::Result<()>;
}

impl EntriesStuff for Entries {
    fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed serializing entries.")
    }

    fn deserialize(json: &str) -> Self {
        let x: Entries = serde_json::from_str(json).expect("Failed deserializing entries.");
        x
    }

    fn save(&self, filename: &std::path::PathBuf) -> std::io::Result<()> {
        std::fs::write(filename, self.serialize())
    }

    fn load(filename: &std::path::PathBuf) -> Self {
        let bytes = &std::fs::read(filename).expect("Failed reading file.");
        let string = std::str::from_utf8(bytes)
            .expect("Failed casting from bytes to a string. I'm not sure I need to do this, even, but I wanted to keep going after I got it to work.");
        Self::deserialize(string)
    }
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

impl Clone for Entry {
    fn clone(&self) -> Entry {
        Entry{
            name: self.name.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            notes: self.notes.clone(),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
        && self.username == other.username
        && self.password == other.password
        && self.notes == other.notes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static S: fn(&'static str)->String = String::from;

    #[test]
    fn entry_clone_and_eq() {
        let original = Entry{
            name: S("name"),
            username: S("username"),
            password: S("password"),
            notes: S("notes"),
        };
        assert_eq!(original, original.clone());
    }

    #[test]
    fn entries_serialize_and_deserialize() {
        let a: Entries = Entries::new();
        let atext = "{}";
        assert_eq!(a.serialize(), atext);
        assert_eq!(Entries::deserialize(&a.serialize()), a);

        let b = a.update(S("First"), Entry{
            name: S("First"),
            username: S("First Username"),
            password: S("First Password"),
            notes: S("First Notes"),
        });
        let btext = r###"{"First":{"name":"First","username":"First Username","password":"First Password","notes":"First Notes"}}"###;
        assert_eq!(b.serialize(), btext);
        assert_eq!(Entries::deserialize(&b.serialize()), b);

        // I chose 2nd instead of Second because I wanted to ensure it was
        // serialized in order.
        let c = b.update(S("2nd"), Entry{
            name: S("2nd"),
            username: S("2nd Username"),
            password: S("2nd Password"),
            notes: S("2nd Notes"),
        });
        let ctext = r###"{"2nd":{"name":"2nd","username":"2nd Username","password":"2nd Password","notes":"2nd Notes"},"First":{"name":"First","username":"First Username","password":"First Password","notes":"First Notes"}}"###;
        assert_eq!(c.serialize(), ctext);
        assert_eq!(Entries::deserialize(&c.serialize()), c);
    }

    #[test]
    fn entries_save_and_load() {
        let original = Entries::new().update(S("first"), Entry{
            name: S("First"),
            username: S("First Username"),
            password: S("First Password"),
            notes: S("First Notes"),
        }).update(S("second"), Entry{
            name: S("Second"),
            username: S("Second Username"),
            password: S("Second Password"),
            notes: S("Second Notes"),
        });

        let filename = mktemp::Temp::new_file().unwrap().to_path_buf();
        original.save(&filename).expect("Failed saving.");

        let loaded = Entries::load(&filename);
        assert_eq!(original, loaded);
    }
}

use std::{thread, time};
use std::io::{stdin, stdout};

use structopt::StructOpt;
use clipboard::{ClipboardContext,ClipboardProvider};

use pm::EntriesStuff;

#[derive(Debug, StructOpt)]
#[structopt(name = "pm", about = "A password manager.", author="Mason Staugler<@mqsoh>")]
struct Opts {
    #[structopt(parse(from_os_str))]
    filename: std::path::PathBuf,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "list")]
    List,
    #[structopt(name = "add")]
    Add,
    #[structopt(name = "show")]
    Show { entry: String, },
    #[structopt(name = "edit")]
    Edit { entry: String, },
    #[structopt(name = "delete")]
    Delete { entry: String },
    #[structopt(name = "clip")]
    Clip { entry: String },
}

pub fn run() {
    let opts = Opts::from_args();
    let entries = open(&mut stdin().lock(), &mut stdout().lock(), &opts.filename)
        .expect("Failed loading entries.");
    match opts.command {
        Command::List => {
            list(&mut stdout().lock(), entries);
        },
        Command::Add => {
            add(&mut stdin().lock(), &mut stdout().lock(), entries)
                .save(&opts.filename)
                .expect("Error saving.");
        },
        Command::Show { entry: entry_name } => {
            match entries.get(&entry_name) {
                Some(_entry) => {
                    show(&mut stdin().lock(), &mut stdout().lock(), entries, &entry_name)
                        .save(&opts.filename)
                        .expect("Error saving.");
                },
                None => {
                    match entry_name.parse::<usize>() {
                        Err(_) => eprintln!("There's no entry by that name or index."),
                        Ok(index) => {
                            match entries.clone().keys().nth(index - 1) {
                                None => eprintln!("There's no entry at that index."),
                                Some(name) => {
                                    show(&mut stdin().lock(), &mut stdout().lock(), entries, name)
                                        .save(&opts.filename)
                                        .expect("Error saving.");
                                }
                            }
                        }
                    }
                }
            }
        },
        Command::Edit { entry: entry_name } => {
            match entries.get(&entry_name) {
                // The entry given was a name that's in the map.
                Some(_entry) => {
                    edit(&mut stdin().lock(), &mut stdout().lock(), entries, &entry_name)
                        .save(&opts.filename)
                        .expect("Error saving.");
                },
                // Maybe it was a number from the list.
                None => {
                    match entry_name.parse::<usize>() {
                        Err(_) => eprintln!("There's no entry by that name or index."),
                        Ok(index) => {
                            // -1 because when we print them they're one-based.
                            // Also, the clone is fine (I think) because of
                            // using the immutable data structures.
                            match entries.clone().keys().nth(index - 1) {
                                None => eprintln!("There's no entry at that index."),
                                Some(name) => {
                                    // Unwrap is safe because the entry must be
                                    // there because I got the name from
                                    // enumerating the entries.
                                    edit(&mut stdin().lock(), &mut stdout().lock(), entries, name)
                                        .save(&opts.filename)
                                        .expect("Error saving.");
                                }
                            }
                        }
                    }
                }
            }
        },
        Command::Delete { entry: entry_name } => {
            match entries.get(&entry_name) {
                Some(_entry) => {
                    delete(&mut stdin().lock(), &mut stdout().lock(), entries, &entry_name)
                        .save(&opts.filename)
                        .expect("Error saving.");
                },
                None => {
                    match entry_name.parse::<usize>() {
                        Err(_) => eprintln!("There's no entry by that name or index."),
                        Ok(index) => {
                            match entries.clone().keys().nth(index - 1) {
                                None => eprintln!("There's no entry at that index."),
                                Some(name) => {
                                    delete(&mut stdin().lock(), &mut stdout().lock(), entries, name)
                                        .save(&opts.filename)
                                        .expect("Error saving.");
                                }
                            }
                        }
                    }
                }
            }
        },
        Command::Clip { entry: entry_name } => {
            match entries.get(&entry_name) {
                Some(_entry) => {
                    clip(&mut stdin().lock(), &mut stdout().lock(), entries, &entry_name);
                    println!("The password will be deleted out of your clipboard in 10 seconds.");
                    thread::sleep(time::Duration::from_secs(10));
                },
                None => {
                    match entry_name.parse::<usize>() {
                        Err(_) => eprintln!("There's no entry by that name or index."),
                        Ok(index) => {
                            match entries.clone().keys().nth(index - 1) {
                                None => eprintln!("There's no entry at that index."),
                                Some(name) => {
                                    clip(&mut stdin().lock(), &mut stdout().lock(), entries, name);
                                    println!("The password will be deleted out of your clipboard in 10 seconds.");
                                    thread::sleep(time::Duration::from_secs(10));
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}

// Prints a prompt for user input and returns the user input (leading and
// trailing whitespace removed).
//
// Not sure why I need this readline<...> thing. Putting the types directly on
// the arguments says that the size can't be determined at compile time.
//
//     ...
//     14 |     readline(std::io::stdin().lock(), std::io::stdout().lock(), "WHAT> ");
//        |              ^^^^^^^^^^^^^^^^^^^^^^^ expected trait std::io::BufRead, found struct `std::io::StdinLock`
//        |
//        = note: expected type `(dyn std::io::BufRead + 'static)`
//                   found type `std::io::StdinLock<'_>`
//     ...
//     19 | fn readline(mut reader: std::io::BufRead, mut writer: std::io::Write, prompt: &str) -> String {
//        |             ^^^^^^^^^^ doesn't have a size known at compile-time
//        |
//        = help: the trait `std::marker::Sized` is not implemented for `(dyn std::io::BufRead + 'static)`
//        = note: to learn more, visit <https://doc.rust-lang.org/book/second-edition/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait>
//        = note: all local variables must have a statically known size
//        = help: unsized locals are gated as an unstable feature
fn readline(reader: &mut impl std::io::BufRead, writer: &mut impl std::io::Write, prompt: &str) -> String {
    writeln!(writer, "{}", prompt)
        .expect("Failed writing output. I can't imagine why this would happen.");
    writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
    let mut input = String::new();
    reader.read_line(&mut input).ok().expect("Failed reading!");
    input.trim().to_owned()
}

// Opens a password file, prompting to initialize one if the given file doesn't
// exist.
fn open<'a>(reader: &mut impl std::io::BufRead, writer: &mut impl std::io::Write, filename: &std::path::PathBuf) -> Result<pm::Entries, &'a str> {
    if filename.exists() {
        Ok(pm::Entries::load(filename))
    } else {
        let answer = readline(reader, writer, &format!(r###"The file "{}" doesn't exist. Create it? (y/n) "###, filename.to_str().expect("Failed to stringify filename.")));
        if answer != "y" {
            Err("They apparently don't want to create a new file!")
        } else {
            let new = pm::Entries::new();
            new.save(filename).expect("Failed saving new, empty entries file.");
            Ok(new)
        }
    }
}

// Lists the entries for the user.
fn list(writer: &mut impl std::io::Write, entries: pm::Entries) {
    for (i, name) in entries.keys().enumerate() {
        // + 1 because I don't want a 0 entry.
        writeln!(writer, "{}: {}", i + 1, name)
            .expect("Failed writing output. I can't imagine why this would happen.");
        writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
    }
}

// Prompts the user to provide the details for a new entry. Checks if the given
// name is unique.
fn add(reader: &mut impl std::io::BufRead, writer: &mut impl std::io::Write, entries: pm::Entries) -> pm::Entries {
    let name = {
        loop {
            let name = readline(reader, writer, "Name: ");
            if entries.contains_key(&name) {
                writeln!(writer, "An entry with the name \"{}\" already exists. Did you want to edit it instead?", name)
                    .expect("Failed writing output. I can't imagine why this would happen.");
                writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
            } else {
                break name;
            }
        }
    };
    let username = readline(reader, writer, "Username: ");
    let password = {
        loop {
            let password = readline(reader, writer, "Password (TODO; generate password, provide options, etc): ");
            if password == "" {
                // TODO; use generated password, or generate a new one and reprompt
            } else {
                break password
            }
        }
    };
    let notes = readline(reader, writer, "Notes: ");
    entries.update(name.clone(), pm::Entry{name, username, password, notes})
}

// Show an entry.
fn show(_reader: &mut impl std::io::BufRead, writer: &mut impl std::io::Write, entries: pm::Entries, name: &String) -> pm::Entries {
    match entries.get(name) {
        None => {
            writeln!(writer, "There's no entry with the name \"{}\".", name)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
            entries
        },
        Some(entry) => {
            writeln!(writer, "Name: {}", entry.name)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writeln!(writer, "Username: {}", entry.username)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writeln!(writer, "Password: {}", entry.password)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writeln!(writer, "Notes: {}", entry.notes)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
            entries
        },
    }
}

// Changes an entry.
fn edit(reader: &mut impl std::io::BufRead, writer: &mut impl std::io::Write, entries: pm::Entries, name: &String) -> pm::Entries {
    match entries.get(name) {
        None => {
            writeln!(writer, "There's no entry with the name \"{}\".", name)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
            entries
        },
        Some(entry) => {
            let original_name = &entry.name;
            let name = {
                let given_name = readline(reader, writer, &format!("Name [{}]: ", &entry.name));
                if given_name == "" {
                    entry.name.to_owned()
                } else {
                    given_name
                }
            };
            let username = {
                let given_username = readline(reader, writer, &format!("Username [{}]: ", &entry.username));
                if given_username == "" {
                    entry.username.to_owned()
                } else {
                    given_username
                }
            };
            let password = {
                let given_password = readline(reader, writer, &format!("Password [{}]: ", &entry.password));
                if given_password == "" {
                    entry.password.to_owned()
                } else {
                    given_password
                }
            };
            let notes = {
                let given_notes = readline(reader, writer, &format!("Notes [{}]: ", &entry.notes));
                if given_notes == "" {
                    entry.notes.to_owned()
                } else {
                    given_notes
                }
            };
            // Remove the entry by the original name first because we're
            // editing the entry. If the name is changed, then we don't want to
            // leave the details under the old name.
            entries.without(original_name)
                .update(name.clone(), pm::Entry{name, username, password, notes})
        },
    }
}

// Removes an entry, asking the user for confirmation first.
fn delete(reader: &mut impl std::io::BufRead, writer: &mut impl std::io::Write, entries: pm::Entries, name: &String) -> pm::Entries {
    match entries.get(name) {
        None => {
            writeln!(writer, "There's no entry with the name \"{}\".", name)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
            entries
        },
        Some(_entry) => {
            let are_you_sure = readline(reader, writer, &format!("Are you sure you want to delete \"{}\"? (y/n) ", name));
            if are_you_sure == "y" {
                entries.without(name)
            } else {
                writeln!(writer, "Keeping \"{}\".", name)
                    .expect("Failed writing output. I can't imagine why this would happen.");
                writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
                entries
            }
        },
    }
}

// Copies the password for an entry to the clipboard and also prints the
// username as a reminder. (That happens to me sometimes when I can't user my
// email address as a username.)
fn clip(_reader: &mut impl std::io::BufRead, writer: &mut impl std::io::Write, entries: pm::Entries, name: &String) -> pm::Entries {
    match entries.get(name) {
        None => {
            writeln!(writer, "There's no entry with the name \"{}\".", name)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
        },
        Some(entry) => {
            let mut board: ClipboardContext = ClipboardProvider::new()
                .expect("Failed getting access to the clipboard.");
            board.set_contents(entry.password.clone()).expect("Failed setting clipboard contents.");
            writeln!(writer, "Copied password for \"{}\". Your username is: {}", name, entry.username)
                .expect("Failed writing output. I can't imagine why this would happen.");
            writer.flush().expect("Couldn't flush stdout! I can't imagine why this would happen.");
        },
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    static S: fn(&'static str)->String = String::from;

    #[test]
    fn test_readline() {
        let mut reader = &(b"y\n")[..];
        let mut writer = Vec::new();
        assert_eq!(readline(&mut reader, &mut writer, "any prompt"), "y");
    }

    #[test]
    fn test_open() {
        let mut reader = &(b"y\n")[..];
        let mut writer: Vec<u8> = Vec::new();
        let filename = {
            mktemp::Temp::new_file()
                .expect("Failed to create a temp file.")
                .to_path_buf()
                .to_str()
                .expect("Failed to stringify.")
                .to_owned()
        };
        let path = std::path::PathBuf::from(&filename);
        assert_eq!(path.exists(), false);

        // A new file will be created with empty entries.
        let new_entries = open(&mut reader, &mut writer, &path).unwrap();
        assert_eq!(new_entries.serialize(), "{}");

        // Save a new entry.
        new_entries.update(S("new"), pm::Entry{
            name: S("new"),
            username: S("new username"),
            password: S("new password"),
            notes: S("new notes"),
        }).save(&path).unwrap();

        // Re-open the newly saved file.
        let mut reader = &(b"y\n")[..];
        let mut writer: Vec<u8> = Vec::new();
        let updated_entries = open(&mut reader, &mut writer, &path).unwrap();
        assert_eq!(updated_entries.serialize(), r###"{"new":{"name":"new","username":"new username","password":"new password","notes":"new notes"}}"###);
    }

    #[test]
    fn test_list() {
        let entries = pm::Entries::new().update(
            S("one"),
            pm::Entry{
                name: S("one"),
                username: S("one username"),
                password: S("one password"),
                notes: S("one notes"),
            },
        ).update(
            S("two"),
            pm::Entry{
                name: S("two"),
                username: S("two username"),
                password: S("two password"),
                notes: S("two notes"),
            },
        );

        let mut writer = Vec::new();
        list(&mut writer, entries);
        assert_eq!(std::str::from_utf8(&writer), Ok("1: one\n2: two\n"));
    }

    #[test]
    fn test_add() {
        let mut reader = &(b"myname\nmyusername\nmypassword\nmynotes\n")[..];
        let mut writer: Vec<u8> = Vec::new();
        let entries = add(&mut reader, &mut writer, pm::Entries::new());
        let expected_entries = pm::Entries::new().update(S("myname"), pm::Entry{
            name: S("myname"),
            username: S("myusername"),
            password: S("mypassword"),
            notes: S("mynotes"),
        });
        assert_eq!(entries, expected_entries);

        // Start off repeating "myname" so that I can check that uniquess is
        // enforced.
        let mut reader = &(b"myname\nmysecondname\nmysecondusername\nmysecondpassword\nmysecondnotes\n")[..];
        let new_entries = add(&mut reader, &mut writer, entries);
        let expected_new_entries = expected_entries.update(S("mysecondname"), pm::Entry{
            name: S("mysecondname"),
            username: S("mysecondusername"),
            password: S("mysecondpassword"),
            notes: S("mysecondnotes"),
        });
        assert_eq!(new_entries, expected_new_entries);
    }

    #[test]
    fn test_edit() {
        let mut reader = &(b"\n\n\n\n")[..];
        let mut writer: Vec<u8> = Vec::new();
        let original_entries = pm::Entries::new().update(S("myname"), pm::Entry{
            name: S("myname"),
            username: S("myusername"),
            password: S("mypassword"),
            notes: S("mynotes"),
        });

        // This first update edits and entry but makes no changes by using the
        // default values.
        let first_update = edit(&mut reader, &mut writer, original_entries.clone(), &S("myname"));
        assert_eq!(first_update, original_entries);

        // Edits without changing the name.
        let mut reader = &(b"\nnewusername\nnewpassword\nnewnotes\n")[..];
        let second_update = edit(&mut reader, &mut writer, first_update.clone(), &S("myname"));
        assert_eq!(second_update, pm::Entries::new().update(S("myname"), pm::Entry{
            name: S("myname"),
            username: S("newusername"),
            password: S("newpassword"),
            notes: S("newnotes"),
        }));

        // Update the name and ensure the entry under the old name is deleted.
        let mut reader = &(b"newname\nnewusername\nnewpassword\nnewnotes\n")[..];
        let second_update = edit(&mut reader, &mut writer, first_update.clone(), &S("myname"));
        assert_eq!(second_update, pm::Entries::new().update(S("newname"), pm::Entry{
            name: S("newname"),
            username: S("newusername"),
            password: S("newpassword"),
            notes: S("newnotes"),
        }));
    }

    #[test]
    fn test_delete() {
        let mut reader = &(b"")[..];
        let mut writer: Vec<u8> = Vec::new();
        let entries = pm::Entries::new().update(S("exists"), pm::Entry{
            name: S("exists"),
            username: S("exists username"),
            password: S("exists password"),
            notes: S("exists notes"),
        });

        let wrong_name_entries = delete(&mut reader, &mut writer, entries.clone(), &S("doesn't exist"));
        assert_eq!(wrong_name_entries, entries);

        let mut reader = &(b"n\n")[..];
        let abort_entries = delete(&mut reader, &mut writer, entries.clone(), &S("exists"));
        assert_eq!(abort_entries, entries);

        let mut reader = &(b"y\n")[..];
        let deleted_entries = delete(&mut reader, &mut writer, entries.clone(), &S("exists"));
        assert_eq!(deleted_entries, pm::Entries::new());
    }

    #[test]
    fn test_clip() {
        let mut reader = &(b"")[..];
        let mut writer: Vec<u8> = Vec::new();
        let entries = pm::Entries::new().update(S("myname"), pm::Entry{
            name: S("myname"),
            username: S("myusername"),
            password: S("mypassword"),
            notes: S("mynotes"),
        });

        let returned_entries = clip(&mut reader, &mut writer, entries.clone(), &S("myname"));
        let mut board: ClipboardContext = ClipboardProvider::new().unwrap();
        assert_eq!(returned_entries, entries);
        assert_eq!(board.get_contents().unwrap(), "mypassword");
    }
}

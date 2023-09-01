use cursive::{
    view::Nameable,
    views::{Button, Checkbox, Dialog, EditView, LinearLayout, ListView, TextView},
    With,
};
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Rule {
    // The name of the rule.
    name: String,
    // The directory containing input files.
    input_dir: PathBuf,
    // The directory to which output files should be written.
    output_dir: PathBuf,
    // The extension of the input and output files, e.g., "txt".
    ext: String,
    // Whether the rule is enabled.
    enabled: bool,
}

fn main() {
    let mut siv = cursive::default();

    siv.load_toml(include_str!("style.toml")).unwrap();

    let rules: Rc<RefCell<Vec<Rule>>> = Rc::new(RefCell::new(load_rules()));

    siv.add_layer(
        Dialog::new()
            .title("Rules")
            .content(ListView::new().with(|list| {
                let rules_clone = rules.clone();
                list.add_child(
                    "",
                    LinearLayout::new(cursive::direction::Orientation::Horizontal)
                        .child(TextView::new("Enabled"))
                        .child(TextView::new("Name")),
                );
                for (i, rule) in rules_clone.borrow().iter().enumerate() {
                    let single_rule = rule.clone();
                    let rules_inner_clone = rules_clone.clone();
                    list.add_child(
                        "",
                        LinearLayout::new(cursive::direction::Orientation::Horizontal)
                            .child(Checkbox::new().with_checked(single_rule.enabled).on_change(
                                move |_, checked| {
                                    rules_inner_clone.borrow_mut()[i].enabled = checked;
                                },
                            ))
                            .child(TextView::new(&single_rule.name))
                            .child(Button::new("Edit", |s| s.quit())),
                    );
                }
            }))
            .button("+", {
                let rules_clone = rules.clone();
                move |s| add_rule(s, Rc::clone(&rules_clone))
            })
            .button("Sort", {
                let rules_clone = rules.clone();
                move |_s| sort(rules_clone.borrow().to_vec())
            })
            .button("Save", {
                let rules_clone = rules;
                move |_s| save_rules(rules_clone.borrow().to_vec())
            }),
    );
    siv.run();
}

// Create a function using cursive that creates a window to add a new rule. Pass the cursive to the function
// and then add the rule to the rules vector.
// Make fields to pass in the rule name, input directory, output directory, and extension.
// Make a button to add the rule to the rules vector.
// Make a button to cancel the rule creation.
// Use editview to get the input directory and output directory and name and extension.
// dont Use a checkbox to enable or disable the rule.
// Make a button to add the rule to the rules vector.
// Make a button to cancel the rule creation.
// Use editview to get the input directory and output directory and name and extension.
// Use a checkbox to enable or disable the rule.

fn add_rule(s: &mut cursive::Cursive, rules: Rc<RefCell<Vec<Rule>>>) {
    s.add_layer(
        Dialog::new()
            .title("Add Rule")
            .content(
                LinearLayout::vertical()
                    .child(TextView::new("Name"))
                    .child(EditView::new().with_name("name"))
                    .child(TextView::new("Input Directory"))
                    .child(EditView::new().with_name("input_dir"))
                    .child(TextView::new("Output Directory "))
                    .child(EditView::new().with_name("output_dir"))
                    .child(TextView::new("Extension"))
                    .child(EditView::new().with_name("ext")),
            )
            .button("Add", move |s| {
                rules.borrow_mut().append(&mut vec![Rule {
                    name: s
                        .call_on_name("name", |view: &mut EditView| view.get_content())
                        .unwrap()
                        .to_string(),
                    input_dir: PathBuf::from(
                        s.call_on_name("input_dir", |view: &mut EditView| view.get_content())
                            .unwrap()
                            .to_string(),
                    ),
                    output_dir: PathBuf::from(
                        s.call_on_name("output_dir", |view: &mut EditView| view.get_content())
                            .unwrap()
                            .to_string(),
                    ),
                    ext: s
                        .call_on_name("ext", |view: &mut EditView| view.get_content())
                        .unwrap()
                        .to_string(),
                    enabled: true,
                }]);
                //How to refresh the listview?
                s.pop_layer();
            })
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    );
}

fn load_rules() -> Vec<Rule> {
    let mut rules: Vec<Rule> = vec![];

    // If rules.json exists, read the file contents and parse it into a vector of `Rule` structs
    if let Ok(_file) = fs::File::open("rules.json") {
        let data = fs::read_to_string("rules.json").unwrap();

        // Replace the brackets with spaces, then split on "}," which should split the string on
        // the closing bracket of each rule.
        data.replace(['[', ']'], " ")
            .split_inclusive("},")
            .for_each(|rulestr| {
                rules.append(&mut vec![serde_json::from_str::<Rule>(
                    rulestr.trim().trim_end_matches(','),
                )
                .unwrap()]);
            });
    } else {
        // If rules.json doesn't exist, create it and print a message to the user.
        fs::File::create("rules.json").unwrap();
        println!("Could not find json file. Creating new one")
    }

    rules
}

fn save_rules(rules: Vec<Rule>) {
    // write rules to json file
    let json = rules
        .iter()
        .map(|rule| serde_json::to_string(rule).unwrap())
        .collect::<Vec<_>>()
        .join(",");
    fs::write("rules.json", format!("[{}]", json)).unwrap();
}

fn sort(rules: Vec<Rule>) {
    rules.iter().for_each(|rule| {
        // Skip this rule if it's disabled or if either input or output directory doesn't exist.
        if !rule.enabled || !rule.input_dir.exists() || !rule.output_dir.exists() {
            return;
        }

        // Copy each file in the input directory that has the correct extension.
        std::fs::read_dir(&rule.input_dir)
            .unwrap()
            .for_each(|file| {
                if let Ok(file) = file {
                    // Only copy files with the correct extension.
                    if let Ok(file_name) = file.file_name().into_string() {
                        if file_name.split('.').last().unwrap() == rule.ext {
                            // Copy the file to the output directory.
                            match fs::copy(file.path(), rule.output_dir.join(file.file_name())) {
                                Ok(_) => (),
                                Err(e) => println!("{:?}", e),
                            };
                        }
                    }
                }
            })
    })
}

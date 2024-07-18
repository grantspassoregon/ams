use crate::controls::act;
use crate::table;
use crate::table::Tabular;
use aid::prelude::{Bandage, Clean};
// see https://www.howtocodeit.com/articles/ultimate-guide-rust-newtypes
use derive_more::{Deref, DerefMut};
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, space0};
use nom::combinator::opt;
use nom::sequence::delimited;
use nom::IResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use strum::IntoEnumIterator;
use toml::{Table, Value};
use tracing::{info, trace, warn};
use uuid::Uuid;
use winit::keyboard::ModifiersState;

#[derive(
    Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Deserialize, Serialize,
)]
pub struct Modifiers {
    pub shift_key: bool,
    pub control_key: bool,
    pub alt_key: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_none(&self) -> bool {
        self.shift_key && self.control_key && self.alt_key && self.super_key
    }

    pub fn and(&mut self, other: &Modifiers) {
        if other.shift_key {
            self.shift_key = true;
        }
        if other.control_key {
            self.control_key = true;
        }
        if other.alt_key {
            self.alt_key = true;
        }
        if other.super_key {
            self.super_key = true;
        }
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut mods = String::new();
        if self.super_key {
            mods.push_str("<⌘> + ");
        }
        if self.control_key {
            mods.push_str("<Cr> + ");
        }
        if self.alt_key {
            mods.push_str("<Alt> + ");
        }
        if self.shift_key {
            mods.push_str("<Sh> + ");
        }
        write!(f, "{mods}")
    }
}

impl From<&ModifiersState> for Modifiers {
    fn from(mods: &ModifiersState) -> Self {
        let mut result = Self::new();
        if mods.shift_key() {
            result.shift_key = true;
        }
        if mods.control_key() {
            result.control_key = true;
        }
        if mods.alt_key() {
            result.alt_key = true;
        }
        if mods.super_key() {
            result.super_key = true;
        }
        result
    }
}

impl From<&Option<ModifiersState>> for Modifiers {
    fn from(mods: &Option<ModifiersState>) -> Self {
        let mut m = Modifiers::new();
        if let Some(n) = mods {
            m = n.into();
        }
        m
    }
}

impl From<&Command> for Modifiers {
    fn from(cmd: &Command) -> Self {
        Self::from(cmd.mods)
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Deserialize, Serialize)]
pub struct Command {
    pub key: String,
    pub mods: Modifiers,
}

impl Command {
    pub fn new(key: &str, mods: &ModifiersState) -> Self {
        Self {
            key: key.to_owned(),
            mods: mods.into(),
        }
    }

    pub fn with_modifier(key: &str, mods: &Modifiers) -> Self {
        Self {
            key: key.to_owned(),
            mods: mods.to_owned(),
        }
    }

    pub fn into_mods(input: &str) -> Option<ModifiersState> {
        match input {
            "cr" => Some(ModifiersState::CONTROL),
            "control" => Some(ModifiersState::CONTROL),
            "alt" => Some(ModifiersState::ALT),
            "shift" => Some(ModifiersState::SHIFT),
            "super" => Some(ModifiersState::SUPER),
            _ => None,
        }
    }

    pub fn is_modifier(input: &str) -> IResult<&str, bool> {
        let (input, _) = Self::separator(input)?;
        let (_, mods) = opt(Self::parse_mod)(input)?;
        if let Some(_) = mods {
            Ok((input, true))
        } else {
            Ok((input, false))
        }
    }

    pub fn word(input: &str) -> IResult<&str, &str> {
        let (rem, _) = space0(input)?;
        alphanumeric1(rem)
    }

    pub fn separator(input: &str) -> IResult<&str, bool> {
        let mut separated = false;
        let (rem, _) = space0(input)?;
        let (rem, sep) = opt(tag("+"))(rem)?;
        if let Some(_) = sep {
            separated = true;
        }
        let (rem, _) = space0(rem)?;
        Ok((rem, separated))
    }

    pub fn parse_mod(input: &str) -> IResult<&str, Option<ModifiersState>> {
        let (input, _) = Self::separator(input)?;
        let (rem, bracketed) = delimited(tag("<"), alphanumeric1, tag(">"))(input)?;
        let (rem, _) = Self::separator(rem)?;
        let bracketed = Self::into_mods(bracketed);
        Ok((rem, bracketed))
    }

    pub fn parse_mods(input: &str) -> IResult<&str, Modifiers> {
        let mut modifiers = Modifiers::new();
        let mut i = input;
        let (_, mut more) = Self::is_modifier(i)?;
        while more {
            let (rem, m) = Self::parse_mod(i)?;
            modifiers.and(&Modifiers::from(&m));
            let (_, check) = Self::is_modifier(rem)?;
            more = check;
            i = rem;
        }
        Ok((i, modifiers))
    }

    pub fn parse_str(input: &str) -> IResult<&str, Option<Self>> {
        let (rem, mods) = Self::parse_mods(input)?;
        let (rem, key) = Self::word(rem)?;
        let command = Command::with_modifier(key, &mods);
        Ok((rem, Some(command)))
    }

    pub fn parse_cmd(input: &str) -> Clean<Self> {
        let (rem, opt) = Self::parse_str(input)?;
        if let Some(mut cmd) = opt {
            if cmd.key == cmd.key.to_uppercase() {
                cmd.mods.shift_key = true;
            }
            Ok(cmd)
        } else {
            Err(Bandage::Nom(rem.to_string()))
        }
    }

    pub fn act(&self, trigger: &Command) -> bool {
        self == trigger
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.mods.is_none() {
            let mods = self.mods.to_string();
            write!(f, "{}{}", mods, self.key)
        } else {
            write!(f, "{}", self.key)
        }
    }
}

impl From<&winit::keyboard::NamedKey> for Command {
    fn from(named: &winit::keyboard::NamedKey) -> Self {
        let mods = ModifiersState::default();
        match named {
            winit::keyboard::NamedKey::Enter => Self::new("Enter", &mods),
            winit::keyboard::NamedKey::Escape => Self::new("Escape", &mods),
            winit::keyboard::NamedKey::ArrowLeft => Self::new("ArrowLeft", &mods),
            winit::keyboard::NamedKey::ArrowRight => Self::new("ArrowRight", &mods),
            winit::keyboard::NamedKey::ArrowUp => Self::new("ArrowUp", &mods),
            winit::keyboard::NamedKey::ArrowDown => Self::new("ArrowDown", &mods),
            _ => Self::default(),
        }
    }
}

impl From<&winit::keyboard::Key> for Command {
    fn from(named: &winit::keyboard::Key) -> Self {
        match named {
            winit::keyboard::Key::Named(k) => Self::from(k),
            _ => Self::default(),
        }
    }
}

impl From<&act::NamedAct> for Command {
    fn from(act: &act::NamedAct) -> Self {
        let mods = ModifiersState::default();
        match act {
            act::NamedAct::Enter => Self::new("enter", &mods),
            act::NamedAct::Escape => Self::new("escape", &mods),
            act::NamedAct::ArrowLeft => Self::new("arrow_left", &mods),
            act::NamedAct::ArrowRight => Self::new("arrow_right", &mods),
            act::NamedAct::ArrowUp => Self::new("arrow_up", &mods),
            act::NamedAct::ArrowDown => Self::new("arrow_down", &mods),
            act::NamedAct::Be => Self::new("be", &mods),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum CommandOptions {
    Commands(CommandGroup),
    Acts(Vec<act::Act>),
}

impl CommandOptions {
    pub fn with_act<T: Into<act::Act>>(&mut self, act: T) {
        match self {
            Self::Commands(_) => warn!("Not an Acts variant!"),
            Self::Acts(acts) => acts.push(act.into()),
        }
    }

    pub fn idx(&self) -> usize {
        match self {
            Self::Acts(acts) => acts[0].idx(),
            Self::Commands(_) => 1000,
        }
    }
}

impl PartialOrd for CommandOptions {
    fn partial_cmp(&self, other: &CommandOptions) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CommandOptions {
    fn cmp(&self, other: &CommandOptions) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Commands(cmd), Self::Commands(other_cmd)) => cmd.cmp(other_cmd),
            (Self::Commands(_), Self::Acts(_)) => std::cmp::Ordering::Greater,
            (Self::Acts(acts), Self::Acts(other_acts)) => acts.cmp(other_acts),
            (Self::Acts(_), Self::Commands(_)) => std::cmp::Ordering::Less,
        }
    }
}

impl std::string::ToString for CommandOptions {
    fn to_string(&self) -> String {
        match self {
            Self::Commands(group) => group.name(),
            Self::Acts(acts) => acts[0].to_string(),
        }
    }
}

impl<T: Into<act::Act>> From<T> for CommandOptions {
    fn from(act: T) -> Self {
        let mut acts = Vec::new();
        acts.push(act.into());
        Self::Acts(acts)
    }
}

impl<T: Into<act::Act> + Clone> From<&[T]> for CommandOptions {
    fn from(acts: &[T]) -> Self {
        let a = acts
            .iter()
            .map(|v| v.clone().into())
            .collect::<Vec<act::Act>>();
        Self::Acts(a)
    }
}

impl<T: Into<act::Act> + Clone> From<Vec<T>> for CommandOptions {
    fn from(acts: Vec<T>) -> Self {
        Self::from(&acts[..])
    }
}

impl From<CommandGroup> for CommandOptions {
    fn from(commands: CommandGroup) -> Self {
        Self::Commands(commands)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct CommandList(Vec<Command>);

/// Names a user-defined custom mapping defined in the config toml as base name **id**.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct CommandGroup {
    /// The table name used in the configuration file to identify the command group.
    pub id: String,
    /// Display name for the command window.
    pub name: String,
    /// Trigger associated with the group.
    pub binding: Command,
    /// Intended for hover or reader descriptions.
    pub help: String,
    /// The [`TableView`] uses `row_id` field to track over changes in row ordering.
    pub row_id: Uuid,
}

impl CommandGroup {
    pub fn from_toml(id: &str, value: &Value) -> Option<Self> {
        let mut name = None;
        let mut binding = None;
        let mut help = None;
        trace!("{:#?}", value);
        match value {
            Value::Table(t) => {
                let command_queue = t.keys().map(|k| k.clone()).collect::<Vec<String>>();
                for key in command_queue {
                    trace!("Reading {}", &key);
                    match key.as_ref() {
                        "name" => {
                            if let Value::String(s) = t[&key].clone() {
                                name = Some(s);
                            }
                        }
                        "binding" => {
                            if let Value::String(s) = t[&key].clone() {
                                match Command::parse_cmd(&s) {
                                    Ok(cmd) => binding = Some(cmd),
                                    Err(e) => trace!("Error parsing binding: {}", e.to_string()),
                                }
                            }
                        }
                        "help" => {
                            if let Value::String(s) = t[&key].clone() {
                                help = Some(s);
                            }
                        }
                        _ => {}
                    }
                }
            }
            v => {
                trace!("Command not recognized: {}", v);
            }
        }
        if let Some(name) = name {
            if let Some(binding) = binding {
                if let Some(help) = help {
                    let row_id = Uuid::new_v4();
                    Some(Self {
                        id: id.to_string(),
                        name,
                        binding,
                        help,
                        row_id,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl PartialOrd for CommandGroup {
    fn partial_cmp(&self, other: &CommandGroup) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CommandGroup {
    fn cmp(&self, other: &CommandGroup) -> std::cmp::Ordering {
        let self_id = self.name();
        let other_id = other.name();
        self_id.cmp(&other_id)
    }
}

impl table::Columnar for CommandGroup {
    fn values(&self) -> Vec<String> {
        let command = self.binding.to_string();
        let act = self.name.clone();
        vec![command, act]
    }

    fn id(&self) -> Uuid {
        self.row_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum CommandMode {
    Normal(ChoiceMap),
}

impl CommandMode {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn choices(&self) -> &ChoiceMap {
        match self {
            Self::Normal(choices) => choices,
        }
    }
}

impl Default for CommandMode {
    fn default() -> Self {
        match ChoiceMap::with_config() {
            Ok(choices) => Self::Normal(choices),
            Err(e) => {
                trace!("Error loading choice map: {}", e.to_string());
                Self::Normal(ChoiceMap::new())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Deserialize, Serialize)]
pub struct Choices(pub HashMap<Command, CommandOptions>);

impl Choices {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(&mut self) -> Clean<()> {
        let cmds = act::NamedAct::iter().map(|v| Command::from(&v));
        let acts = act::NamedAct::iter();
        cmds.zip(acts)
            .map(|(c, a)| self.0.insert(c, a.into()))
            .for_each(drop);

        Ok(())
    }

    pub fn from_toml<T: Clone + std::str::FromStr>(value: &Value) -> Clean<Self> {
        use std::str::FromStr;
        trace!("{:#?}", value);
        match value {
            Value::Table(t) => {
                let mut choices = HashMap::new();
                let command_queue = t.keys().map(|k| k.clone()).collect::<Vec<String>>();
                for key in command_queue {
                    trace!("Reading {}", &key);
                    if let Value::String(s) = &value[&key] {
                        let s = s.to_owned();
                        let command = Command::parse_cmd(&s)?;
                        trace!("Command result: {:#?}", &command);
                        match act::Act::from_str(&key) {
                            Ok(act) => {
                                let opts = CommandOptions::from(vec![act]);
                                choices.insert(command, opts);
                            }
                            Err(_) => {
                                info!("Command not recognized.");
                            }
                        }
                        // let act = T::from_str(&key)?;
                        // if let Some(a) = act {
                        //     let opts = CommandOptions::from(vec![a]);
                        //     choices.insert(command, opts);
                        // }
                    }
                }
                Ok(Self(choices))
            }
            v => {
                trace!("Command not recognized: {}", v);
                Err(Bandage::Hint("Could not read from toml.".to_string()))
            }
        }
    }

    /// If any of the base names defined in the config toml map to an [`Act`], and the value
    /// associated with the name parses to a valid ['Command'], then it returns a [`Choices`]
    /// containing the name/value pair.
    pub fn try_from_toml(value: &Value) -> Option<Self> {
        let mut choices = Choices::new();
        if let Ok(entry) = Self::from_toml::<act::AppAct>(value) {
            choices.extend(entry.0.into_iter());
        }
        if let Ok(entry) = Self::from_toml::<act::EguiAct>(value) {
            choices.extend(entry.0.into_iter());
        }
        if let Ok(entry) = Self::from_toml::<act::NamedAct>(value) {
            choices.extend(entry.0.into_iter());
        }
        if choices.is_empty() {
            None
        } else {
            Some(choices)
        }
    }

    /// Attempt to read a [`CommandGroup`] from toml and insert it into the HashMap of choices.
    pub fn command_group(&mut self, value: &Value) -> Clean<()> {
        trace!("{:#?}", value);
        match value {
            Value::Table(t) => {
                let command_queue = t.keys().map(|k| k.clone()).collect::<Vec<String>>();

                for key in command_queue {
                    trace!("Reading {}", &key);
                    let group = CommandGroup::from_toml(&key, &t[&key]);
                    if let Some(cmds) = group {
                        self.0
                            .insert(cmds.binding.clone(), CommandOptions::from(cmds.clone()));
                        trace!("Added {}", cmds.name);
                    }
                }
            }
            v => {
                trace!("Command not recognized: {}", v);
            }
        }

        Ok(())
    }

    // pub fn value(&self) -> &HashMap<Command, CommandOptions> {
    //     match self {
    //         Self(data) => data,
    //     }
    // }
    //
    // pub fn value_mut(&mut self) -> &mut HashMap<Command, CommandOptions> {
    //     match self {
    //         Self(data) => data,
    //     }
    // }
}

impl Default for Choices {
    fn default() -> Self {
        // Choices::with_config().unwrap()
        let hm = HashMap::new();
        Self(hm)
    }
}

/// A context map relating different groups of keyboard mappings with an associated command
/// trigger.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ChoiceMap(pub HashMap<String, Choices>);

impl ChoiceMap {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_toml(value: &Value) -> Option<Self> {
        let mut choice_map = ChoiceMap::new();
        trace!("{:#?}", value);
        match value {
            Value::Table(t) => {
                let keys = t.keys().map(|k| k.clone()).collect::<Vec<String>>();
                for key in keys {
                    if let Some(c) = Choices::try_from_toml(&t[&key]) {
                        choice_map.0.insert(key, c);
                    }
                }
            }
            v => {
                trace!("Choices not recognized: {}", v);
            }
        }
        if choice_map.0.is_empty() {
            None
        } else {
            Some(choice_map)
        }
    }

    pub fn with_config() -> Clean<Self> {
        let config = include_bytes!("../../config.toml");
        trace!("Config read: {} u8.", config.len());
        let stringly = String::from_utf8_lossy(config);
        let config = stringly.parse::<Table>().unwrap();
        trace!("Config read: {}", config);
        let mut choice_map = ChoiceMap::new();
        let groups = &config["groups"];
        if let Some(c) = ChoiceMap::from_toml(groups) {
            choice_map.0.extend(c.0);
        }
        let commands = &config["commands"];
        choice_map.command_group(&commands)?;
        trace!("Choices: {:#?}", choice_map);
        Ok(choice_map)
    }

    pub fn command_group(&mut self, value: &Value) -> Clean<()> {
        trace!("{:#?}", value);
        match value {
            Value::Table(t) => {
                let command_queue = t.keys().map(|k| k.clone()).collect::<Vec<String>>();

                for key in command_queue {
                    trace!("Reading {}", &key);
                    if let Some(_) = self.0.get(&key) {
                        let group = CommandGroup::from_toml(&key, &t[&key]);
                        if let Some(cmds) = group {
                            if let Some(normal) = self.0.get_mut("normal") {
                                normal
                                    .0
                                    .insert(cmds.binding.clone(), CommandOptions::from(cmds));
                            }
                        }
                    }
                }
            }
            v => {
                trace!("Command not recognized: {}", v);
            }
        }

        Ok(())
    }
}

/// The `CommandRow` struct represents a choice from [`Choices`] as a table row for display.
/// The `CommandRow` struct implements the [`Columnar`] trait for use in a [`TableView`].
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct CommandRow {
    /// The `id` field holds a [`Uuid`] for use by the [`TableView`].
    id: Uuid,
    /// The `command` field is the string representation of a command.
    command: String,
    /// The `act` field is the string representation of an act or command group.
    act: String,
    /// The `visible` field is set by checking the "Show" box in a [`TableView`].
    visible: bool,
    /// The `active` field indicates the command is in the active view.
    active: bool,
}

impl CommandRow {
    pub fn new(command: &str, act: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            command: command.to_string(),
            act: act.to_string(),
            visible: true,
            active: true,
        }
    }
}

impl table::Columnar for CommandRow {
    fn values(&self) -> Vec<String> {
        vec![self.command.clone(), self.act.clone()]
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

/// The `CommandTable` struct is a wrapper around a vector of type [`CommandRow`].  The
/// `CommandTable` implements the [`Tabular`] trait for display in a [`TableView`], and implements
/// [`Filtration`] by *bool* to control visibility of commands in the command window.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deref,
    DerefMut,
    Deserialize,
    Serialize,
)]
pub struct CommandTable(Vec<CommandRow>);

impl CommandTable {
    pub fn rows_mut(&mut self) -> &mut Vec<CommandRow> {
        &mut self.0
    }
}

impl table::Tabular<CommandRow> for CommandTable {
    fn headers() -> Vec<String> {
        vec!["Command".to_string(), "Act".to_string()]
    }
    fn rows(&self) -> Vec<CommandRow> {
        self.0.clone()
    }

    fn sort_by_col(&mut self, column_index: usize, reverse: bool) {
        match column_index {
            0 => {
                if reverse {
                    self.0.sort_by(|a, b| b.command.cmp(&a.command));
                } else {
                    self.0.sort_by(|a, b| a.command.cmp(&b.command));
                }
            }
            1 => {
                if reverse {
                    self.0.sort_by(|a, b| b.act.cmp(&a.act));
                } else {
                    self.0.sort_by(|a, b| a.act.cmp(&b.act));
                }
            }
            _ => {
                tracing::info!("Column index not recognized.");
            }
        }
    }
}

impl table::Filtration<CommandTable, bool> for CommandTable {
    fn filter(&mut self, filter: &bool) -> Self {
        let mut rows = self.to_vec();
        rows.retain(|v| v.visible == *filter);
        Self(rows)
    }
}

impl From<&Choices> for CommandTable {
    fn from(choices: &Choices) -> Self {
        let rows = choices
            .0
            .iter()
            .map(|(k, v)| CommandRow::new(&k.to_string(), &v.to_string()))
            .collect::<Vec<CommandRow>>();
        CommandTable(rows)
    }
}

impl From<&ChoiceMap> for CommandTable {
    fn from(choice_map: &ChoiceMap) -> Self {
        let mut rows = Vec::new();
        for (_, choices) in &choice_map.0 {
            let table = Self::from(choices);
            rows.extend(table.0);
        }
        Self(rows)
    }
}

impl From<&CommandMode> for CommandTable {
    fn from(mode: &CommandMode) -> Self {
        match mode {
            CommandMode::Normal(choice_map) => Self::from(choice_map),
        }
    }
}
// pub command_view: TableView<CommandTable, CommandRow, bool>,
// /// Lookup keys for the [`ChoiceMap`] passed down from the global state.
// pub command_keys: Option<(String, Option<Command>)>,
// /// Active [`ChoiceMap`] from the `command` field of [`State`].
// pub command_tree: CommandMode,

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommandView {
    /// Window showing available commands.
    pub table: table::TableView<CommandTable, CommandRow, bool>,
    // /// Lookup keys for the [`ChoiceMap`] passed down from the global state.
    // pub key: Option<String>,
    // /// Lookup command for the [`Choices`] within the [`ChoiceMap`].
    // pub command: Option<Command>,
    /// Active [`ChoiceMap`] from the `command` field of [`State`].
    pub data: CommandTable,
    /// The `option` field indicates if the view options show be visible.
    pub options: bool,
    /// The `refresh` field is set as a flag when the options change to reload the table.
    pub refresh: Option<()>,
}

impl CommandView {
    pub fn check_options(&mut self) {
        if let Some(()) = self.refresh.take() {
            // rebuild the table with or without check boxes
            match self.options {
                true => {
                    // with check boxes
                    let config = table::TableConfig::new().checked();
                    // record current state of checks
                    let checks = self.table.checks.clone();
                    // create a new table view by cloning the original data
                    self.table = table::TableView::with_config(self.data.clone(), config);
                    // return checks to previous state
                    self.table.checks = checks;
                    tracing::trace!("Table reset.");
                }
                false => {
                    // record current state of checks
                    let checks = self.table.checks.clone();
                    // without check boxes
                    self.table = table::TableView::new(self.data.clone());
                    // return checks to previous state
                    self.table.checks = checks;
                    // filter data by whether visible
                    let mut table = self.table.data.clone();
                    let table = table::Filtration::filter(&mut table, &true);
                    // set the view to the filtered table
                    let view = self.table.view_mut();
                    *view = table;
                    tracing::trace!("Table filtered and reset!");
                }
            }
        }
    }

    pub fn set_view(&mut self, from: &CommandTable) {
        tracing::trace!("Setting view.");
        // receive command table from the lens
        // for each row in the `from` table
        // mark the corresponding row in self active
        let from_rows = from.rows();
        for row in self.data.rows_mut() {
            for from_row in &from_rows {
                if row.command == from_row.command && row.act == from_row.act {
                    row.active = true;
                }
            }
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.check_options();
        self.table.table(ui);
        if ui.checkbox(&mut self.options, "Show options").changed() {
            match self.options {
                // Activating checks
                true => {
                    // Copy the data to read the visibility.
                    let rows = self.data.clone();
                    // Create a mutable reference to the checks field to record visibility.
                    let checks = self.table.checks_mut();
                    // For each row, set the check to match the visibility
                    for row in rows.iter() {
                        if let Some(check) = checks.get_mut(&row.id) {
                            *check = row.visible;
                        }
                    }
                    tracing::trace!("Checks set from data.");
                }
                // Deactivating checks
                false => {
                    // Copy the current status of checks to write to row state
                    let checks = self.table.checks().clone();
                    // for each row, copy checks to visible to record any changes by the user
                    for row in self.data.iter_mut() {
                        if let Some(check) = checks.get(&row.id) {
                            row.visible = *check;
                        }
                    }
                    tracing::trace!("Data set from checks.");
                }
            }
            self.refresh = Some(());
        }
    }
}

impl From<&CommandTable> for CommandView {
    fn from(table: &CommandTable) -> Self {
        let data = table.clone();
        let table = table::TableView::new(data.clone());
        let refresh = Some(());
        Self {
            table,
            data,
            refresh,
            ..Default::default()
        }
    }
}

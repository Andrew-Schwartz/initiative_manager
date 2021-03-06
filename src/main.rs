// ignored on other targets
#![windows_subsystem = "windows"]

#![warn(clippy::pedantic)]
// @formatter:off
#![allow(
clippy::too_many_lines,
clippy::default_trait_access,
clippy::wildcard_imports,
clippy::module_name_repetitions,
clippy::cast_precision_loss,
clippy::cast_possible_truncation,
clippy::cast_sign_loss,
clippy::cast_lossless,
clippy::cast_possible_wrap,
)]
// @formatter:on

#![feature(array_windows)]
#![feature(array_chunks)]

use std::fmt::Display;
use std::fs;
use std::fs::{FileType, OpenOptions};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use iced::*;
use iced::tooltip::Position;
use iced_aw::{Icon, ICON_FONT};
use iced_native::Event;
use itertools::Itertools;
use once_cell::sync::Lazy;
use rand::Rng;
use self_update::cargo_crate_version;
use serde::{Deserialize, Serialize};

use crate::style::{SettingsBarStyle, Style};
use crate::utils::{censor_name, checkbox, Hidden, Hp, MakeHidden, SpacingExt, Tap, TextInputState, ToggleButtonState, TooltipExt};

#[macro_use]
mod utils;
mod style;
mod hotkey;
mod update;

static SAVE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let path = dirs::data_local_dir().unwrap_or_default()
        .join("initiative_manager");
    std::fs::create_dir_all(&path).unwrap();
    path
});
static PARTY_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let path = SAVE_DIR.clone()
        .join("party");
    std::fs::create_dir_all(&path).unwrap();
    path
});
static ENCOUNTER_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let path = SAVE_DIR.clone()
        .join("encounters");
    std::fs::create_dir_all(&path).unwrap();
    path
});

#[derive(Debug)]
struct Entity {
    name: Hidden<String>,
    remove_state: button::State,
    hp: Hidden<u32>,
    damage: TextInputState,
    heal: TextInputState,
    reaction_free: ToggleButtonState,
    concentrating: ToggleButtonState,
    legendary_actions: Option<Hidden<(u32, u32)>>,
    la_minus: button::State,
    la_plus: button::State,
    initiative: Hidden<u32>,
    init_up: button::State,
    init_down: button::State,
}

impl Entity {
    fn new(name: Hidden<String>, hp: Hidden<u32>, initiative: Hidden<u32>) -> Self {
        Self {
            name,
            remove_state: Default::default(),
            hp,
            damage: Default::default(),
            heal: Default::default(),
            reaction_free: ToggleButtonState::new(true),
            concentrating: ToggleButtonState::new(false),
            legendary_actions: Default::default(),
            la_minus: Default::default(),
            la_plus: Default::default(),
            initiative,
            init_up: Default::default(),
            init_down: Default::default(),
        }
    }
}

#[derive(Default)]
struct NewEntity {
    name: Hidden<TextInputState>,
    init: Hidden<TextInputState>,
    hp: Hidden<TextInputState>,
    leg_acts: Hidden<TextInputState>,
}

#[derive(Deserialize, Serialize)]
struct Pc {
    name: String,
    hp: u32,
}

#[derive(Deserialize, Serialize)]
struct Enemy {
    name: Hidden<String>,
    hp: Hidden<u32>,
    legendary_actions: Option<Hidden<u32>>,
    initiative: Hidden<u32>,
}

enum SaveMode {
    None,
    SaveEncounter(TextInputState, button::State),
    DeleteEncounter(String, TextInputState, button::State),
    LoadEncounter(String, button::State, scrollable::State, Vec<Enemy>),
    SaveParty(TextInputState, button::State),
    DeleteParty(String, TextInputState, button::State),
    LoadParty(String, button::State, scrollable::State, Vec<(Pc, TextInputState)>),
}

impl SaveMode {
    fn view(&mut self, style: Style) -> Element<Message> {
        match self {
            SaveMode::None => Space::new(Length::Shrink, Length::Shrink).into(),
            SaveMode::SaveEncounter(text, button) => {
                let savable = !text.content.is_empty();
                let encounter_name = text.text_input("Encounter Name", Message::EncounterName)
                    .style(style)
                    .tap_if(savable, |text| text.on_submit(Message::SaveEncounter));
                let submit = Button::new(button, Text::new("Submit").size(16))
                    .style(style)
                    .tap_if(savable, |btn| btn.on_press(Message::SaveEncounter));
                Row::new()
                    .align_items(Align::Center)
                    .push(encounter_name)
                    .push_space(8)
                    .push(submit)
                    .into()
            }
            SaveMode::DeleteEncounter(name, text, button) => {
                let matches = text.content == *name;
                let encounter_name = text.text_input("Delete", Message::EncounterName)
                    .style(style)
                    .tap_if(matches, |txt| txt.on_submit(Message::DeleteEncounter(name.clone())));
                let submit = Button::new(
                    button,
                    Text::new(format!("Type '{name}' to confirm")).size(16),
                ).style(style)
                    .tap_if(matches, |btn| btn.on_press(Message::DeleteEncounter(name.clone())));
                Row::new()
                    .align_items(Align::Center)
                    .push(encounter_name)
                    .push_space(8)
                    .push(submit)
                    .into()
            }
            SaveMode::LoadEncounter(name, submit, scroll, enemies) => {
                let submit = Button::new(
                    submit,
                    Text::new("Confirm"),
                ).style(style)
                    .on_press(Message::LoadEncounter(name.clone()));

                let [names, hps, las, inits] = enemies.into_iter()
                    .enumerate()
                    .fold(["Name (Hidden)", "HP (Hidden)", "Leg. Acts. (Hidden)", "Initiative (Hidden)"].map(|title| vec![Element::from(Text::new(title))]),
                          |[mut names, mut hps, mut las, mut inits], (idx, Enemy { name, hp, legendary_actions, initiative })| {
                              fn view<T: Display>(Hidden(t, hidden): &Hidden<T>, idx: usize, part: HideablePart, style: Style) -> Element<'static, Message> {
                                  let hide = checkbox(*hidden, move |hidden| Message::EncounterHide(idx, hidden, part))
                                      .style(style)
                                      .size(16);
                                  let row = Row::new()
                                      .push(Text::new(format!("{t} (")).size(16))
                                      .push(hide)
                                      .push(Text::new(')').size(16));
                                  row.into()
                              }

                              names.push(view(&name, idx, HideablePart::Name, style));
                              // let name = Text::new(format!("{name} ({})", if *hidden { '???' } else { '???' })).size(16);
                              // names.push(name.into());

                              hps.push(view(&hp, idx, HideablePart::Hp, style));
                              // let hp = Text::new(hp.to_string()).size(16);
                              // hps.push(hp.into());

                              if let Some(la) = legendary_actions {
                                  las.push(view(&la, idx, HideablePart::LegActs, style));
                                  // let la = Text::new(roman::to(*la as _).unwrap()).size(16);
                                  // las.push(la.into());
                              }

                              inits.push(view(&initiative, idx, HideablePart::Initiative, style));
                              // let init = Text::new(initiative.to_string()).size(16);
                              // inits.push(init.into());

                              [names, hps, las, inits]
                          });
                let table = Scrollable::new(scroll)
                    .push(Row::new()
                        .push(Column::with_children(names).spacing(5))
                        .push_space(Length::Fill)
                        .push(Column::with_children(hps).spacing(5))
                        .tap_if(las.len() > 1, |row| row
                            .push_space(Length::Fill)
                            .push(Column::with_children(las).spacing(5)))
                        .push_space(Length::Fill)
                        .push(Column::with_children(inits).spacing(5))
                    );

                Column::new()
                    .align_items(Align::Center)
                    .push(submit)
                    .push_space(7)
                    .push(table)
                    .into()
            }
            SaveMode::SaveParty(text, button) => {
                let savable = !text.content.is_empty();
                let party_name = text.text_input("Party Name", Message::PartyName)
                    .style(style)
                    .tap_if(savable, |txt| txt.on_submit(Message::SaveParty));
                let submit = Button::new(button, Text::new("Submit"))
                    .style(style)
                    .tap_if(savable, |btn| btn.on_press(Message::SaveParty));
                Row::new()
                    .align_items(Align::Center)
                    .push(party_name)
                    .push_space(8)
                    .push(submit)
                    .into()
            }
            SaveMode::DeleteParty(name, text, button) => {
                let matches = text.content == *name;
                let party_name = text.text_input("Delete", Message::PartyName)
                    .style(style)
                    .tap_if(matches, |txt| txt.on_submit(Message::DeleteParty(name.clone())));
                let submit = Button::new(
                    button,
                    Text::new(format!("Type '{name}' to confirm"))
                        .size(16),
                ).style(style)
                    .tap_if(matches, |btn| btn.on_press(Message::DeleteParty(name.clone())));
                Row::new()
                    .align_items(Align::Center)
                    .push(party_name)
                    .push_space(8)
                    .push(submit)
                    .into()
            }
            SaveMode::LoadParty(party_name, button, scroll, rows) => {
                let all_entered = rows.iter().all(|(_, txt)| !txt.content.is_empty());
                let button = Button::new(button, Text::new("Submit Initiatives"))
                    .style(style)
                    .tap_if(all_entered, |b| b.on_press(Message::LoadParty(party_name.clone())));

                let (names, inits) = rows.iter_mut()
                    .enumerate()
                    .fold(
                        (Column::new().align_items(Align::Start).spacing(5), Column::new().align_items(Align::End).spacing(5)),
                        |(names, inits), (i, (pc, text))| {
                            let names = names.push(Text::new(&pc.name));
                            let text = text.text_input("Initiative", move |str| Message::PcInitiative(i, str))
                                .style(style)
                                .tap_if(all_entered, |txt| txt.on_submit(Message::LoadParty(party_name.clone())));
                            let inits = inits.push(text);
                            (names, inits)
                        },
                    );
                let scrollable = Scrollable::new(scroll)
                    .push(Row::new().push(names).push_space(12).push(inits));

                Column::new()
                    .align_items(Align::Center)
                    .push(button)
                    .push_space(10)
                    .push(scrollable)
                    .into()
            }
        }
    }
}

impl Default for SaveMode {
    fn default() -> Self {
        Self::None
    }
}

pub struct InitiativeManager {
    update_state: UpdateState,
    update_url: String,
    dm_view: ToggleButtonState,
    style: Style,
    width: u32,
    height: u32,
    style_button: button::State,
    entities: Vec<Entity>,
    highlight_state: Option<(usize, container::Style)>,
    scroll: scrollable::State,
    new_entity_submit: button::State,
    new_entity: NewEntity,
    turn: usize,
    next_turn: button::State,
    prev_turn: button::State,
    save_encounter: button::State,
    delete_encounter: pick_list::State<String>,
    load_encounter: pick_list::State<String>,
    save_party: button::State,
    delete_party: pick_list::State<String>,
    load_party: pick_list::State<String>,
    save_mode: SaveMode,
}

#[derive(Debug, Clone)]
pub enum Message {
    Update(update::Message),
    ToggleVisibility,
    ToggleStyle,
    Resize(u32, u32),
    ToggleHidden(usize, HideablePart),
    DeleteEntity(usize),
    EditDamage(usize, String),
    Damage(usize),
    HighlightConcentration(usize, Instant),
    EditHealing(usize, String),
    Heal(usize),
    Reaction(usize),
    Concentrate(usize),
    LegActionMinus(usize),
    LegActionPlus(usize),
    MoveUp(usize),
    MoveDown(usize),
    NewName(String),
    NewInit(String),
    NewHp(String),
    NewLas(String),
    NewHidden(bool, HideablePart),
    NewEntitySubmit,
    HotKey(hotkey::Message),
    NextTurn,
    PrevTurn,
    SaveEncounter,
    EncounterName(String),
    DeleteEncounter(String),
    LoadEncounter(String),
    EncounterHide(usize, bool, HideablePart),
    SaveParty,
    PartyName(String),
    DeleteParty(String),
    LoadParty(String),
    PcInitiative(usize, String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum HideablePart {
    Name,
    Hp,
    LegActs,
    Initiative,
}

impl Application for InitiativeManager {
    type Executor = iced_futures::executor::Tokio;
    type Message = Message;
    type Flags = (u32, u32);

    fn new((width, height): Self::Flags) -> (Self, Command<Message>) {
        let window = Self {
            update_state: UpdateState::Checking,
            update_url: "".to_string(),
            dm_view: ToggleButtonState::new_with(true, [Icon::EyeSlashFill, Icon::EyeFill]),
            style: Default::default(),
            width,
            height,
            style_button: Default::default(),
            entities: vec![],
            highlight_state: None,
            scroll: Default::default(),
            new_entity_submit: Default::default(),
            new_entity: Default::default(),
            turn: 0,
            next_turn: Default::default(),
            prev_turn: Default::default(),
            save_encounter: Default::default(),
            delete_encounter: Default::default(),
            load_encounter: Default::default(),
            save_party: Default::default(),
            delete_party: Default::default(),
            load_party: Default::default(),
            save_mode: Default::default(),
        };
        let command = async {
            // wait briefly to so that loading doesn't take so long
            tokio::time::sleep(Duration::from_millis(500)).await;
            Message::Update(update::Message::CheckForUpdate)
        }.into();
        (window, command)
    }

    fn title(&self) -> String {
        "Initiatives".into()
    }

    fn update(&mut self, message: Self::Message, _: &mut iced::Clipboard) -> Command<Message> {
        let mut commands = Vec::new();
        match message {
            Message::Update(msg) => if let Err(e) = update::handle(self, msg) {
                self.update_state = UpdateState::Errored(e.to_string());
            },
            Message::ToggleVisibility => self.dm_view.invert(),
            Message::ToggleStyle => self.style = !self.style,
            Message::Resize(width, height) => {
                self.width = width;
                self.height = height;
            }
            Message::ToggleHidden(i, part) => {
                let entity = &mut self.entities[i];
                match part {
                    HideablePart::Name => entity.name.1 = !entity.name.1,
                    HideablePart::Hp => entity.hp.1 = !entity.hp.1,
                    HideablePart::LegActs => { entity.legendary_actions.as_mut().map(|las| las.1 = !las.1); }
                    HideablePart::Initiative => entity.initiative.1 = !entity.initiative.1,
                }
            }
            Message::DeleteEntity(i) => {
                self.entities.remove(i);
                if i < self.turn {
                    self.turn -= 1;
                }
            }
            Message::EditDamage(i, damage) => {
                if damage.parse::<u32>().is_ok() || damage.is_empty() {
                    self.entities[i].damage.content = damage;
                }
            }
            Message::Damage(i) => {
                let entity = &mut self.entities[i];
                let damage = &mut entity.damage.content;
                if !damage.is_empty() {
                    entity.hp.0 = entity.hp.0.saturating_sub(damage.parse().unwrap());
                    damage.clear();
                    if entity.concentrating.value {
                        commands.push(async move {
                            Message::HighlightConcentration(i, Instant::now() + Duration::from_millis(1400))
                        }.into());
                    }
                }
            }
            Message::HighlightConcentration(i, highlight_done) => {
                let now = Instant::now();
                if highlight_done > now {
                    self.highlight_state = Some((i, container::Style {
                        text_color: {
                            let millis = highlight_done.duration_since(now).as_millis();
                            let r = 1.0 - (millis % 700) as f32 / 1400.0;
                            Some(Color::new(r, 0.0, 0.0, 1.0))
                        },
                        background: Color::TRANSPARENT.into(),
                        ..Box::<dyn container::StyleSheet>::from(self.style).style()
                    }));
                    commands.push(async move {
                        tokio::time::sleep(Duration::from_millis(15)).await;
                        Message::HighlightConcentration(i, highlight_done)
                    }.into())
                } else {
                    self.highlight_state = None;
                }
            }
            Message::EditHealing(i, healing) => {
                if healing.parse::<u32>().is_ok() || healing.is_empty() {
                    self.entities[i].heal.content = healing;
                }
            }
            Message::Heal(i) => {
                let entity = &mut self.entities[i];
                let heal = &mut entity.heal.content;
                if !heal.is_empty() {
                    entity.hp.0 += heal.parse::<u32>().unwrap();
                    heal.clear();
                }
            }
            Message::Reaction(i) => self.entities[i].reaction_free.invert(),
            Message::Concentrate(i) => self.entities[i].concentrating.invert(),
            Message::LegActionMinus(i) => {
                if let Some(Hidden((_, left), _)) = &mut self.entities[i].legendary_actions {
                    *left -= 1;
                }
            }
            Message::LegActionPlus(i) => {
                if let Some(Hidden((_, left), _)) = &mut self.entities[i].legendary_actions {
                    *left += 1;
                }
            }
            Message::MoveUp(i) => self.entities.swap(i, i - 1),
            Message::MoveDown(i) => self.entities.swap(i, i + 1),
            Message::NewName(name) => self.new_entity.name.0.content = name,
            Message::NewInit(init) => {
                if init.is_empty() || init == "-" || init == "+" || init.parse::<i32>().is_ok() {
                    self.new_entity.init.0.content = init;
                }
            }
            Message::NewHp(hp) => {
                if hp.is_empty() || hp.parse::<Hp>().is_ok() {
                    self.new_entity.hp.0.content = hp;
                }
            }
            Message::NewLas(las) => {
                if las.is_empty() || las.parse::<u32>().is_ok() {
                    self.new_entity.leg_acts.0.content = las;
                }
            }
            Message::NewHidden(hidden, part) => match part {
                HideablePart::Name => self.new_entity.name.1 = hidden,
                HideablePart::Hp => self.new_entity.hp.1 = hidden,
                HideablePart::LegActs => self.new_entity.leg_acts.1 = hidden,
                HideablePart::Initiative => self.new_entity.init.1 = hidden,
            },
            Message::NewEntitySubmit => {
                if !self.new_entity.name.0.content.is_empty() {
                    let NewEntity {
                        name: Hidden(TextInputState { content: name, .. }, name_hidden),
                        init: Hidden(TextInputState { content: init, .. }, init_hidden),
                        hp: Hidden(TextInputState { content: hp, .. }, hp_hidden),
                        leg_acts: Hidden(TextInputState { content: leg_acts, .. }, leg_acts_hidden),
                    } = std::mem::take(&mut self.new_entity);
                    let hp = if hp.is_empty() {
                        Hp::new(0)
                    } else { hp.parse().unwrap() }
                        .into_number()
                        .unwrap_or(0);
                    let init = if init.is_empty() || init.starts_with(['+', '-']) {
                        let modifier = init.parse().unwrap_or(0);
                        let roll = rand::thread_rng().gen_range(1..=20);
                        std::cmp::max(0, roll + modifier) as u32
                    } else {
                        init.parse().unwrap()
                    };
                    let mut entity = Entity::new(
                        Hidden(name, name_hidden),
                        Hidden(hp, hp_hidden),
                        Hidden(init, init_hidden),
                    );
                    if !leg_acts.is_empty() {
                        let leg_acts = leg_acts.parse().unwrap();
                        if leg_acts != 0 {
                            entity.legendary_actions = Some((leg_acts, leg_acts).hidden(leg_acts_hidden));
                        }
                    }
                    Self::insert_entity(&mut self.entities, &mut self.turn, entity)
                }
            }
            Message::HotKey(hotkey) => match hotkey {
                hotkey::Message::NextField(forwards) => {
                    // todo add other set of states for player inits
                    let cycle = |states: &mut [&mut text_input::State]| {
                        if let Some(i) = states.into_iter().position(|state| state.is_focused()) {
                            if forwards {
                                states[i].unfocus();
                                states[(i + 1) % states.len()].focus();
                            } else if !forwards {
                                states[i].unfocus();
                                states[if i == 0 { states.len() - 1 } else { i - 1 }].focus();
                            }
                        }
                    };
                    cycle(&mut [
                        &mut self.new_entity.name.0.state,
                        &mut self.new_entity.init.0.state,
                        &mut self.new_entity.hp.0.state,
                        &mut self.new_entity.leg_acts.0.state,
                    ]);
                    match &mut self.save_mode {
                        SaveMode::LoadParty(_, _, _, rows) => {
                            let mut vec = rows.into_iter()
                                .map(|(_, text_input)| &mut text_input.state)
                                .collect_vec();
                            cycle(&mut vec);
                        }
                        _ => {}
                    }
                }
            }
            Message::NextTurn => {
                self.turn = (self.turn + 1).checked_rem(self.entities.len()).unwrap_or(0);
                if let Some(entity) = self.entities.get_mut(self.turn) {
                    entity.reaction_free.value = true;
                    if let Some(Hidden((tot, left), _)) = &mut entity.legendary_actions {
                        *left = *tot;
                    }
                }
            }
            Message::PrevTurn => self.turn = if self.turn == 0 {
                self.entities.len().saturating_sub(1)
            } else {
                self.turn.saturating_sub(1)
            },
            Message::SaveEncounter => {
                match &mut self.save_mode {
                    SaveMode::SaveEncounter(name, _) if !name.content.is_empty() => {
                        let enemies = self.entities.iter()
                            .map(|Entity { name, hp, initiative, legendary_actions, .. }| Enemy {
                                name: name.clone(),
                                hp: *hp,
                                legendary_actions: legendary_actions.map(|Hidden((las, _), hidden)| Hidden(las, hidden)),
                                initiative: *initiative,
                            }).collect_vec();
                        let file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(ENCOUNTER_DIR.join(format!("{}.json", name.content)))
                            .unwrap();
                        serde_json::to_writer(file, &enemies).unwrap();

                        self.save_mode = SaveMode::None;
                    }
                    other => *other = SaveMode::SaveEncounter(TextInputState::focused(), Default::default()),
                }
            }
            Message::EncounterName(name) => match &mut self.save_mode {
                SaveMode::SaveEncounter(state, _)
                | SaveMode::DeleteEncounter(_, state, _) => {
                    state.content = name;
                }
                _ => {}
            }
            Message::DeleteEncounter(name) => {
                match &mut self.save_mode {
                    SaveMode::DeleteEncounter(curr_name, _, _) if name == *curr_name => {
                        // ignore error
                        let _ = fs::remove_file(ENCOUNTER_DIR.join(format!("{name}.json")));

                        self.save_mode = SaveMode::None;
                    }
                    other => *other = SaveMode::DeleteEncounter(name, TextInputState::focused(), Default::default())
                }
            }
            Message::LoadEncounter(name) => {
                // rows to enter initiative for each character
                match &mut self.save_mode {
                    SaveMode::LoadEncounter(curr_name, _, _, rows) if name == *curr_name => {
                        rows.drain(0..)
                            .map(|Enemy { name, hp, legendary_actions, initiative }| {
                                Entity::new(name, hp, initiative)
                                    .tap_if_some(legendary_actions, |mut e, Hidden(las, hidden)| {
                                        e.legendary_actions = Some(Hidden((las, las), hidden));
                                        e
                                    })
                            }).for_each(|e| Self::insert_entity(&mut self.entities, &mut self.turn, e));

                        self.save_mode = SaveMode::None;
                    }
                    other => {
                        let file = OpenOptions::new()
                            .read(true)
                            .open(ENCOUNTER_DIR.join(format!("{name}.json")))
                            .unwrap();
                        let rows = serde_json::from_reader::<_, Vec<Enemy>>(file)
                            .unwrap()
                            .into_iter()
                            .collect();
                        *other = SaveMode::LoadEncounter(name, Default::default(), Default::default(), rows)
                    }
                }
            }
            Message::EncounterHide(idx, hide, part) => match &mut self.save_mode {
                SaveMode::LoadEncounter(_, _, _, enemies) => match part {
                    HideablePart::Name => enemies[idx].name.1 = hide,
                    HideablePart::Hp => enemies[idx].hp.1 = hide,
                    HideablePart::LegActs => if let Some(las) = &mut enemies[idx].legendary_actions {
                        las.1 = hide;
                    },
                    HideablePart::Initiative => enemies[idx].initiative.1 = hide,
                }
                _ => {}
            },
            Message::SaveParty => {
                // create name field, once submitted save names and HP of all entities
                match &mut self.save_mode {
                    SaveMode::SaveParty(name, _) if !name.content.is_empty() => {
                        let pcs = self.entities.iter()
                            .map(|Entity { name, hp, .. }| Pc { name: name.0.clone(), hp: hp.0 })
                            .collect_vec();
                        let file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(PARTY_DIR.join(format!("{}.json", name.content)))
                            .unwrap();
                        serde_json::to_writer(file, &pcs).unwrap();

                        self.save_mode = SaveMode::None;
                    }
                    other => *other = SaveMode::SaveParty(TextInputState::focused(), Default::default()),
                };
            }
            Message::PartyName(name) => match &mut self.save_mode {
                SaveMode::SaveParty(state, _)
                | SaveMode::DeleteParty(_, state, _) => {
                    state.content = name;
                }
                _ => {}
            },
            Message::DeleteParty(name) => {
                match &mut self.save_mode {
                    SaveMode::DeleteParty(curr_name, _, _) if name == *curr_name => {
                        // ignore error
                        let _ = fs::remove_file(PARTY_DIR.join(format!("{name}.json")));

                        self.save_mode = SaveMode::None;
                    }
                    other => *other = SaveMode::DeleteParty(name, TextInputState::focused(), Default::default())
                }
            }
            Message::LoadParty(name) => {
                // rows to enter initiative for each character
                match &mut self.save_mode {
                    SaveMode::LoadParty(curr_name, _, _, rows) if name == *curr_name => {
                        rows.drain(0..)
                            .map(|(Pc { name, hp }, txt)| {
                                Entity::new(name.hidden(false), hp.hidden(false), Hidden(txt.content.parse().unwrap(), false))
                            }).for_each(|e| Self::insert_entity(&mut self.entities, &mut self.turn, e));

                        self.save_mode = SaveMode::None;
                    }
                    other => {
                        let file = OpenOptions::new()
                            .read(true)
                            .open(PARTY_DIR.join(format!("{name}.json")))
                            .unwrap();
                        let mut rows: Vec<_> = serde_json::from_reader::<_, Vec<Pc>>(file)
                            .unwrap()
                            .into_iter()
                            .map(|pc| (pc, TextInputState::default()))
                            .collect();
                        if let Some((_, TextInputState { state, .. })) = rows.first_mut() {
                            state.focus();
                        }
                        *other = SaveMode::LoadParty(name, Default::default(), Default::default(), rows)
                    }
                }
            }
            Message::PcInitiative(idx, init) => if let SaveMode::LoadParty(_, _, _, rows) = &mut self.save_mode {
                if init.is_empty() || init.parse::<u32>().is_ok() {
                    rows[idx].1.content = init;
                }
            },
        };
        Command::batch(commands)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let listeners = iced_native::subscription::events_with(|event, _status| {
            match event {
                Event::Keyboard(e) => hotkey::handle(e),
                Event::Window(e) => match e {
                    iced_native::window::Event::Resized { width, height } => Some(Message::Resize(width, height)),
                    iced_native::window::Event::FileDropped(path) => {
                        println!("path = {:?}", path);
                        todo!()
                    }
                    _ => None,
                },
                // Event::Mouse(e) => hotmouse::handle(e),
                // Event::Touch(_) => None,
                _ => None
            }
        });
        match &self.update_state {
            UpdateState::Ready | UpdateState::Downloading(_) => {
                let download = Subscription::from_recipe(update::Download { url: self.update_url.clone() })
                    .map(|p| Message::Update(update::Message::Progress(p)));
                Subscription::batch([
                    listeners,
                    download,
                ])
            }
            _ => listeners
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        const INITIATIVES_PADDING: u16 = 8;
        const INITIATIVES_BORDER_PADDING: u16 = 4;
        const INITIATIVES_INTERIOR_PADDING: u16 = 4;
        const CONTROL_SPACING: u16 = 5;
        const HP_MOD_WIDTH: u16 = 26;
        const COLUMN_WIDTH_RATIO: (u16, u16) = (3, 2);

        let dm_view = self.dm_view.value;
        let style = self.style;
        let width = self.width;
        let init_width = (width as u16 * COLUMN_WIDTH_RATIO.0) as f64 / (COLUMN_WIDTH_RATIO.0 + COLUMN_WIDTH_RATIO.1) as f64;
        let options_width = width as f64 - init_width;

        let has_legendary_action = self.entities.iter()
            .any(|e| e.legendary_actions.is_some());

        let spacing_w = 1.0;
        let name_w = 5.0;
        let hp_w = 3.0;
        let reaction_w = 4.0;
        let conc_w = 4.0;
        let leg_acts_w = if has_legendary_action { 5.0 } else { 0.0 };
        let initiative_w = 4.0;
        let num_spaces = (3 + has_legendary_action as u32) as f64;
        let denominator = spacing_w * num_spaces + name_w + hp_w + reaction_w + conc_w + leg_acts_w + initiative_w;

        let spacing_w = init_width * spacing_w / denominator;
        let name_w = init_width * name_w / denominator;
        let hp_w = init_width * hp_w / denominator;
        let reaction_w = init_width * reaction_w / denominator;
        let conc_w = init_width * conc_w / denominator;
        let leg_acts_w = init_width * leg_acts_w / denominator;
        let initiative_w = init_width * initiative_w / denominator;

        let n_entities = self.entities.len();
        let turn = self.turn;

        let mut up_down = vec![false];
        up_down.extend(
            self.entities.array_windows::<2>()
                .map(|[a, b]| a.initiative.0 == b.initiative.0)
                .flat_map(|bool| [bool, bool])
        );
        up_down.push(false);
        let up_down = up_down.array_chunks::<2>().collect_vec();

        let (end, start) = self.entities.split_at_mut(turn);
        let highlight = self.highlight_state.map(|(mut idx, style)| {
            idx = (idx as isize - turn as isize).wrapping_rem_euclid(n_entities as _) as _;
            (idx, style)
        });

        let scrollable = start.iter_mut()
            .chain(end.iter_mut())
            .enumerate()
            .fold(
                Scrollable::new(&mut self.scroll)
                    .align_items(Align::Center)
                    .push(Container::new(
                        Row::new()
                            .align_items(Align::Center)
                            .spacing(spacing_w as _)
                            .push(Text::new("Name")
                                .size(17)
                                .width(Length::Units(name_w as _)))
                            .push(Text::new("HP")
                                .size(17)
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .width(Length::Units(hp_w as _)))
                            .push(Text::new("Reaction Free")
                                .size(17)
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .width(Length::Units(reaction_w as _)))
                            .push(Text::new("Concentrating")
                                .size(17)
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .width(Length::Units(conc_w as _)))
                            .tap_if(has_legendary_action, |row| row
                                .push(Text::new("Legendary Actions ")
                                    .size(17)
                                    .horizontal_alignment(HorizontalAlignment::Center)
                                    .width(Length::Units(leg_acts_w as _))))
                            .push(Text::new("Initiative")
                                .size(17)
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .width(Length::Units(initiative_w as u16)))
                    )
                        .padding(INITIATIVES_INTERIOR_PADDING)
                        .style(style.initiative_table(1))),
                |col, (i, Entity {
                    name,
                    // censored_name,
                    remove_state,
                    hp,
                    damage,
                    heal,
                    reaction_free,
                    concentrating,
                    legendary_actions,
                    la_minus,
                    la_plus,
                    initiative,
                    init_up,
                    init_down,
                })| {
                    let idx = (i + turn) % n_entities;
                    // let hidden = hidden_toggle.value;
                    // let is_visible = !hidden || dm_view;
                    let style = style.initiative_table(i);

                    // let hide_entity_button = hidden_toggle.button_with(|text| text.size(16))
                    //     .style(style)
                    //     .on_press(Message::ToggleHidden(idx));
                    let name = Button::new(
                        remove_state, Text::new(if dm_view || !name.1 {
                            name.0.to_string()
                        } else {
                            // censored_name.clone()
                            censor_name(&name.0)
                        }).size(16),
                    ).style(style)
                        .padding(0)
                        .width(Length::Fill)
                        .on_press(Message::DeleteEntity(idx));
                    let name = Container::new(
                        Row::new()
                            .align_items(Align::Center)
                            // .tap_if(!dm_view, |row| row
                            //     .push(hide_entity_button)
                            //     .push_space(5))
                            .push(name))
                        .align_x(Align::Start)
                        .style(style);

                    let hp = Text::new(if dm_view || !hp.1 {
                        hp.0.to_string()
                    } else {
                        "??".to_string()
                    }).horizontal_alignment(HorizontalAlignment::Right)
                        .size(16);
                    let damage = damage.text_input(
                        "damage",
                        move |s| Message::EditDamage(idx, s),
                    ).style(style)
                        .size(9)
                        .width(Length::Units(HP_MOD_WIDTH))
                        .on_submit(Message::Damage(idx));
                    let heal = heal.text_input(
                        "heal",
                        move |s| Message::EditHealing(idx, s),
                    ).style(style)
                        .size(9)
                        .width(Length::Units(HP_MOD_WIDTH))
                        .on_submit(Message::Heal(idx));
                    let hp_mods = Column::new()
                        .align_items(Align::Start)
                        .push(damage)
                        .push(heal);
                    let hp = Container::new(
                        Row::new()
                            .align_items(Align::Center)
                            .push(hp
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .width(Length::Shrink))
                            .tap_if(dm_view, |row| row
                                .push_space(CONTROL_SPACING)
                                .push(hp_mods.width(Length::Shrink)))
                    )
                        .style(style)
                        .align_x(Align::Center);

                    let reaction = reaction_free.button()
                        .style(style)
                        .on_press(Message::Reaction(idx));

                    let conc = concentrating.button_with(|txt| {
                        let mut cont = Container::new(txt)
                            .align_x(Align::Center)
                            .style(style);
                        match highlight {
                            Some((idx, style)) if idx == i => {
                                struct ContainerStyle(container::Style);
                                impl container::StyleSheet for ContainerStyle {
                                    fn style(&self) -> container::Style {
                                        self.0
                                    }
                                }
                                cont = cont.style(ContainerStyle(style));
                            }
                            _ => {}
                        };
                        cont
                    })
                        .style(style)
                        .on_press(Message::Concentrate(idx));

                    let legendary_actions = if let Some(Hidden((tot, left), _)) = legendary_actions {
                        let mut minus = Button::new(la_minus, Text::new(" - ").size(16))
                            .padding(0)
                            .style(style);
                        if *left != 0 {
                            minus = minus.on_press(Message::LegActionMinus(idx));
                        }
                        let mut plus = Button::new(la_plus, Text::new(" + ").size(16))
                            .padding(0)
                            .style(style);
                        if *left != *tot {
                            plus = plus.on_press(Message::LegActionPlus(idx));
                        }
                        Row::new()
                            .spacing(2)
                            .align_items(Align::Center)
                            .push(minus)
                            .push(Text::new(roman::to(*left as _).unwrap_or_else(String::new)).size(16))
                            .push(plus)
                    } else {
                        Row::new()
                    };
                    let legendary_actions = Container::new(legendary_actions)
                        .style(style)
                        .align_x(Align::Center);

                    let &[move_up, move_down] = up_down[idx];
                    // let initiative = Text::new(format!("{} ({})", initiative, tiebreaker));
                    let initiative = Text::new(initiative.0.to_string())
                        .size(16)
                        .horizontal_alignment(HorizontalAlignment::Left);
                    let mut up = Button::new(
                        init_up,
                        if move_up {
                            Text::new(Icon::ArrowUp).font(ICON_FONT).size(8)
                                .horizontal_alignment(HorizontalAlignment::Left)
                        } else {
                            Text::new(" ").size(8)
                                .horizontal_alignment(HorizontalAlignment::Left)
                        },
                    ).style(style)
                        .padding(0);
                    if move_up {
                        up = up.on_press(Message::MoveUp(idx));
                    }
                    let mut down = Button::new(
                        init_down,
                        if move_down {
                            Text::new(Icon::ArrowDown).font(ICON_FONT).size(8)
                                .horizontal_alignment(HorizontalAlignment::Left)
                        } else {
                            Text::new(" ").size(8)
                                .horizontal_alignment(HorizontalAlignment::Left)
                        },
                    ).style(style)
                        .padding(0);
                    if move_down {
                        down = down.on_press(Message::MoveDown(idx));
                    }
                    let init_mods = Column::new()
                        .push(up)
                        .push_space(5)
                        .push(down)
                        .align_items(Align::Start);
                    let initiative = Container::new(
                        Row::new()
                            .push(initiative
                                .horizontal_alignment(HorizontalAlignment::Center)
                                .width(Length::Shrink))
                            .push_space(CONTROL_SPACING)
                            .push(init_mods.width(Length::Shrink))
                    )
                        .style(style)
                        .align_x(Align::Center);

                    col.push(Container::new(
                        Row::new()
                            .align_items(Align::Center)
                            .push(name
                                .width(Length::Units(name_w as _)))
                            .push_space(Length::Units(spacing_w as _))
                            .push(hp
                                .width(Length::Units(hp_w as u16 + CONTROL_SPACING)))
                            .push_space(Length::Units(spacing_w as _))
                            .push(reaction
                                .width(Length::Units(reaction_w as _)))
                            .push_space(Length::Units(spacing_w as _))
                            .push(conc
                                .width(Length::Units(conc_w as _)))
                            .tap_if(has_legendary_action, |row| row
                                .push_space(Length::Units(spacing_w as _))
                                .push(legendary_actions
                                    .width(Length::Units(leg_acts_w as _))))
                            .push_space(Length::Units(spacing_w as _))
                            .push(initiative
                                .width(Length::Units(initiative_w as u16 + CONTROL_SPACING)))
                    )
                        .padding(INITIATIVES_INTERIOR_PADDING)
                        .style(style))
                });

        let initiatives = Container::new(
            Container::new(scrollable)
                .padding(INITIATIVES_BORDER_PADDING)
                .style(style.initiative_table_border())
                .center_x()
        ).padding(INITIATIVES_PADDING)
            .center_x();

        let next = Button::new(
            &mut self.next_turn,
            Text::new("Next Turn"),
        ).style(style)
            .on_press(Message::NextTurn);

        let prev = Button::new(
            &mut self.prev_turn,
            Text::new("Previous Turn"),
        ).style(style)
            .on_press(Message::PrevTurn);

        let next_btns = Row::new()
            .push_space(Length::FillPortion(2))
            .push(next)
            .push_space(Length::Fill)
            .push(prev)
            .push_space(Length::FillPortion(2));

        let new_ready = {
            let hp_empty = self.new_entity.hp.0.content.is_empty();
            let hp_parses = self.new_entity.hp.0.content.parse::<Hp>()
                .ok()
                .and_then(|hp| hp.into_number())
                .is_some();
            let hp_ready = hp_empty || hp_parses;
            let name_ready = !self.new_entity.name.0.content.is_empty();
            hp_ready && name_ready
        };

        let submit_new_button = Button::new(
            &mut self.new_entity_submit,
            Text::new("Submit"),
        ).style(style)
            .tap_if(new_ready,
                    |btn| btn.on_press(Message::NewEntitySubmit));

        let hide_msg = |part| move |hide| Message::NewHidden(hide, part);

        let new_name = self.new_entity.name.0.text_input(
            "Name",
            Message::NewName,
        ).style(style)
            .tap_if(new_ready,
                    |txt| txt.on_submit(Message::NewEntitySubmit));
        let hide = Checkbox::new(
            self.new_entity.name.1,
            "Hide?",
            hide_msg(HideablePart::Name),
        ).style(style);
        let new_name = Row::new()
            .push(new_name.width(Length::FillPortion(2)))
            .push_space(Length::Fill)
            .push(hide);

        // should display a d20 somehow if you put like +3 (it'll roll)
        let new_init = self.new_entity.init.0.text_input(
            "init or ??mod",
            Message::NewInit,
        ).style(style)
            .tap_if(new_ready,
                    |txt| txt.on_submit(Message::NewEntitySubmit));
        let hide = Checkbox::new(
            self.new_entity.init.1,
            "Hide?",
            hide_msg(HideablePart::Initiative),
        ).style(style);
        let new_init = Row::new()
            .push(new_init.width(Length::FillPortion(2)))
            .push_space(Length::Fill)
            .push(hide);

        let new_hp = self.new_entity.hp.0.text_input(
            "hp",
            Message::NewHp,
        ).style(style)
            .tap_if(new_ready,
                    |txt| txt.on_submit(Message::NewEntitySubmit));
        let hide = Checkbox::new(
            self.new_entity.hp.1,
            "Hide?",
            hide_msg(HideablePart::Hp),
        ).style(style);
        let new_hp = Row::new()
            .push(new_hp.width(Length::FillPortion(2)))
            .push_space(Length::Fill)
            .push(hide);

        let new_las = self.new_entity.leg_acts.0.text_input(
            "# of legendary actions",
            Message::NewLas,
        ).style(style)
            .tap_if(new_ready,
                    |txt| txt.on_submit(Message::NewEntitySubmit));
        let hide = Checkbox::new(
            self.new_entity.leg_acts.1,
            "Hide?",
            hide_msg(HideablePart::LegActs),
        ).style(style);
        let new_las = Row::new()
            .push(new_las.width(Length::FillPortion(2)))
            .push_space(Length::Fill)
            .push(hide);

        let save_encounter = Button::new(
            &mut self.save_encounter,
            Text::new("Save Encounter").size(14),
        ).style(style)
            .on_press(Message::SaveEncounter);

        // let start = Instant::now();
        let encounters = fs::read_dir(&*ENCOUNTER_DIR).unwrap()
            .flatten()
            .filter(|entry| entry.file_type().ok().filter(FileType::is_file).is_some())
            .map(|entry| entry.path().file_stem().unwrap().to_string_lossy().into_owned())
            .collect_vec();
        // println!("read encounters = {:?}", start.elapsed());

        let delete_encounter = PickList::new(
            &mut self.delete_encounter,
            encounters.clone(),
            Some(String::from("Delete Encounter")),
            Message::DeleteEncounter,
        ).style(style)
            .text_size(14);

        let load_encounter = PickList::new(
            &mut self.load_encounter,
            encounters,
            Some(String::from("Load Encounter")),
            Message::LoadEncounter,
        ).style(style)
            .text_size(14);

        let save_party = Button::new(
            &mut self.save_party,
            Text::new("Save Players").size(14),
        ).style(style)
            .on_press(Message::SaveParty);

        // todo store the saved ones and then have it watch the directory for updates
        // let start = Instant::now();
        let parties = fs::read_dir(&*PARTY_DIR).unwrap()
            .flatten()
            .filter(|entry| entry.file_type().ok().filter(FileType::is_file).is_some())
            .map(|entry| entry.path().file_stem().unwrap().to_string_lossy().into_owned())
            .collect_vec();
        // println!("read parties = {:?}", start.elapsed());

        let delete_party = PickList::new(
            &mut self.delete_party,
            parties.clone(),
            Some(String::from("Delete Players")),
            Message::DeleteParty,
        ).style(style)
            .text_size(14);

        let load_party = PickList::new(
            &mut self.load_party,
            parties,
            Some(String::from("Load Players")),
            Message::LoadParty,
        ).style(style)
            .text_size(14);

        let new_entity_col = Container::new(
            Column::new()
                .push(next_btns)
                .push_space(10)
                .push_rule(20)
                .push(Column::new()
                    .align_items(Align::Center)
                    .push(submit_new_button)
                    .push_space(15)
                    .push(new_name)
                    .push_space(6)
                    .push(new_init)
                    .push_space(6)
                    .push(new_hp)
                    .push_space(6)
                    .push(new_las)
                )
                .push_rule(40)
                .push(Container::new(Row::new()
                    .push(Column::new()
                        .push(save_encounter.width(Length::Units((options_width / 3.3) as _)))
                        .push_space(10)
                        .push(save_party.width(Length::Units((options_width / 3.3) as _))))
                    .push_space(Length::Fill)
                    .push(Column::new()
                        .push(delete_encounter.width(Length::Units((options_width / 3.3) as _)))
                        .push_space(10)
                        .push(delete_party.width(Length::Units((options_width / 3.3) as _))))
                    .push_space(Length::Fill)
                    .push(Column::new()
                        .push(load_encounter.width(Length::Units((options_width / 3.3) as _)))
                        .push_space(10)
                        .push(load_party.width(Length::Units((options_width / 3.3) as _))))
                ).width(Length::Shrink))
                .tap_if(
                    !matches!(self.save_mode, SaveMode::None),
                    |col| col.push_space(10).push(self.save_mode.view(style)),
                )
        ).padding(8)
            .center_x();

        let toggle_visibility = self.dm_view.button_with(|text| text.size(12))
            .style(style.settings_bar())
            .on_press(Message::ToggleVisibility)
            .tooltip(if dm_view { "Hide Secret Stats" } else { "Show Secret Stats" }, Position::Top)
            .size(10);

        let toggle_style = Button::new(
            &mut self.style_button,
            Text::new(Icon::BrightnessHigh)
                .font(ICON_FONT)
                .size(12),
        ).style(style.settings_bar())
            .on_press(Message::ToggleStyle)
            .tooltip(format!("Switch to {} theme", !style), Position::Top)
            .size(10);

        let bottom_bar = Container::new(Row::new()
            .spacing(2)
            .push_space(4)
            .push(self.update_state.view(style.settings_bar()))
            .push_space(Length::Fill)
            .push(toggle_visibility)
            .push(toggle_style)
            .height(Length::Units(20))
            .align_items(Align::Center)
        ).style(style.settings_bar())
            .align_y(Align::Center);

        let content = Column::new()
            .push(Row::new()
                .push(initiatives.width(Length::FillPortion(COLUMN_WIDTH_RATIO.0)))
                .push(new_entity_col.width(Length::FillPortion(COLUMN_WIDTH_RATIO.1)))
                .height(Length::Shrink)
            ).push_space(Length::Fill)
            .push(bottom_bar);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .align_y(Align::Start)
            .style(style)
            .into()
    }
}

impl InitiativeManager {
    fn insert_entity(entities: &mut Vec<Entity>, turn: &mut usize, entity: Entity) {
        let index = entities.iter()
            .position(|e| e.initiative.0 < entity.initiative.0)
            .unwrap_or(entities.len());
        entities.insert(index, entity);
        if *turn >= index {
            *turn += 1;
        }
    }
}

fn main() {
    if let Some("TARGET") = std::env::args().nth(1).as_deref() {
        println!("{}", self_update::get_target());
        return;
    }

    let mut size = iced::window::Settings::default().size;
    size.1 = (size.1 as f64 * 0.9) as _;
    <InitiativeManager as iced::Application>::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../resources/arial.ttf")),
        window: iced::window::Settings {
            size,
            min_size: None,
            icon: None,
            ..Default::default()
        },
        flags: size,
        ..Default::default()
    }).unwrap();
}

#[derive(Debug)]
pub enum UpdateState {
    Checking,
    Ready,
    Downloading(f32),
    UpToDate,
    Downloaded,
    Errored(String),
}

impl UpdateState {
    #[must_use]
    pub fn view(&self, style: SettingsBarStyle) -> Element<crate::Message> {
        const VER: &str = cargo_crate_version!();
        match self {
            &Self::Downloading(pct) => {
                Row::new()
                    .align_items(Align::Center)
                    .push(Text::new("Downloading").size(10))
                    .push_space(5)
                    .push(ProgressBar::new(0.0..=100.0, pct)
                        .style(style)
                        .height(Length::Units(12)) // bottom bar is 20 pts
                        .width(Length::Units(100)))
                    .into()
            }
            view_as_text => match view_as_text {
                Self::Checking => Text::new("Checking for updates..."),
                Self::Ready => Text::new("Preparing to download..."),
                Self::Downloaded => Text::new("Downloaded new version! Restart program to get new features!"),
                Self::UpToDate => Text::new(format!("Up to date, v{}", VER)),
                Self::Errored(e) => Text::new(format!("Error downloading new version: {}. Running v{}", e, VER)),
                Self::Downloading(_) => unreachable!(),
            }.size(10).into()
        }
    }
}
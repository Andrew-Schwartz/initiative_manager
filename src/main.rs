use iced::*;

use crate::style::Style;
use crate::utils::SpacingExt;

mod style;
mod utils;

struct Entity {
    name: String,
    remove_state: button::State,
    hp: u32,
    // damage
    reaction_left: bool,
    legendary_actions: Option<u32>,
    initiative: u32,
    tiebreaker: u32,
}

impl Entity {
    fn new(name: String, hp: u32, initiative: u32) -> Self {
        Self {
            name,
            remove_state: Default::default(),
            hp,
            reaction_left: true,
            legendary_actions: None,
            initiative,
            tiebreaker: 0,
        }
    }
}

#[derive(Default)]
struct Window {
    style: Style,
    entities: Vec<Entity>,
    scroll: scrollable::State,
}

#[derive(Debug, Clone)]
enum Message {
    DeleteEntity(usize),
    Reaction(usize, bool),
}

impl Sandbox for Window {
    type Message = Message;

    fn new() -> Self {
        Self {
            style: Default::default(),
            entities: vec![
                Entity::new("TEST1".to_string(), 15, 13),
                Entity::new("TEST2".to_string(), 16, 9),
                Entity::new("TEST3".to_string(), 17, 5),
            ],
            scroll: Default::default(),
        }
    }

    fn title(&self) -> String {
        "Initiatives".into()
    }

    fn update(&mut self, message: Self::Message) {
        println!("message = {:?}", message);
        match message {
            Message::DeleteEntity(i) => { self.entities.remove(i); }
            Message::Reaction(i, state) => self.entities[i].reaction_left = state,
        };
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let style = self.style;

        let scrollable = self.entities.iter_mut()
            .enumerate()
            .fold(
                Scrollable::new(&mut self.scroll)
                    .align_items(Align::Center),
                |col, (i, Entity {
                    name,
                    remove_state,
                    hp,
                    reaction_left,
                    legendary_actions,
                    initiative,
                    tiebreaker
                })| {
                    let style = style.initiative_table(i);

                    let name = Button::new(
                        remove_state, Text::new(name.to_string()),
                    ).style(style)
                        .on_press(Message::DeleteEntity(i));

                    let hp = Text::new(hp.to_string());

                    let reaction = Checkbox::new(
                        *reaction_left, "Reaction",
                        move |reaction| Message::Reaction(i, reaction),
                    );

                    let initiative = Text::new(initiative.to_string());

                    col.push(
                        Row::new()
                            .align_items(Align::Center)
                            .push(name.width(Length::Shrink))
                            .push_space(Length::Fill)
                            .push(hp.width(Length::Shrink))
                            .push_space(Length::Fill)
                            .push(reaction.width(Length::Shrink))
                            .push_space(Length::Fill)
                            .push(initiative.width(Length::Shrink))
                    )
                });

        let initiatives = Container::new(scrollable)
            .padding(5)
            .center_x();

        let new_entity_col = Column::new();

        Container::new(Row::new()
            .push(initiatives.width(Length::FillPortion(3)))
            .push(new_entity_col.width(Length::FillPortion(2)))
        ).width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .align_y(Align::Start)
            .style(style)
            .into()
    }
}

fn main() {
    <Window as iced::Sandbox>::run(Settings {
        antialiasing: true,
        default_font: Some(include_bytes!("../resources/arial.ttf")),
        ..Default::default()
    }).unwrap();
}

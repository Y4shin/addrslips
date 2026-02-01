use iced::{
    Color, Element, Theme, border, widget::{column, container::Style, container, row, text}
};
use iced_widget::container::bordered_box;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    CreateArea,
    DetectAddresses,
    AssignStreets,
    AssignGroups,
}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use Step::*;
        let self_val = match self {
            CreateArea => 0,
            DetectAddresses => 1,
            AssignStreets => 2,
            AssignGroups => 3,
        };
        let other_val = match other {
            CreateArea => 0,
            DetectAddresses => 1,
            AssignStreets => 2,
            AssignGroups => 3,
        };
        self_val.cmp(&other_val)
    }
}

impl Step {
    fn style(self, other: Self) -> impl Fn(&Theme) -> Style {
        move |theme: &Theme| {
            let style = bordered_box(theme).border(border::width(5));
            // if done, gray out background
            if self >= other {
                let mut color_rgba = theme.palette().background.into_rgba8();
                color_rgba[0] /= 2;
                color_rgba[1] /= 2;
                color_rgba[2] /= 2;
                style.background(Color::from_rgb8(color_rgba[0], color_rgba[1], color_rgba[2]))
            } else {
                style.background(theme.palette().background)
            }
        }
    }
}


pub fn layout<'a, Message>(
    sidebar: impl Into<Element<'a, Message>>,
    main_content: impl Into<Element<'a, Message>>,
    step: Step,
) -> Element<'a, Message>
where
    Message: 'a,
{
    container(row![
        container(column![
            container(column![
                container(text("CreateArea")).style(step.style(Step::CreateArea)).padding(10),
                container(text("DetectAddresses")).style(step.style(Step::DetectAddresses)).padding(10),
                container(text("AssignStreets")).style(step.style(Step::AssignStreets)).padding(10),
                container(text("AssignGroups")).style(step.style(Step::AssignGroups)).padding(10),
            ]),
            container(sidebar.into()).height(iced::Length::Fill),
        ]).width(iced::Length::FillPortion(1)),
        container(main_content.into()).width(iced::Length::FillPortion(4)),
    ])
    .center_x(iced::Length::Fill)
    .center_y(iced::Length::Fill)
    .into()
}

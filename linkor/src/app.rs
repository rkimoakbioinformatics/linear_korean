use crate::compile;
use crate::consts::*;
use crate::Args;
use iced::Task;
use std::io::Read;

lazy_static::lazy_static! {
  pub static ref FONT_NAME: std::sync::Arc<std::sync::RwLock<String>> = {
    std::sync::Arc::new(std::sync::RwLock::new("PilGi".to_string()))
  };
}

pub struct App {
    pub args: Args,
    pub test_content: iced::widget::text_editor::Content,
    pub font_loaded: bool,
    pub char_gap: String,
    pub jung_gap: String,
    pub jong_gap: String,
    pub sw_str: String,
    pub text_size_str: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Compile,
    ChangeTtfFile(String),
    CharGap(String),
    JungGap(String),
    JongGap(String),
    TextSize(String),
    StrokeWidth(String),
    Edit(iced::widget::text_editor::Action),
}

impl App {
    pub fn new(args: Args) -> (Self, Task<Message>) {
        let mut f = std::fs::File::open("/Users/rick/Downloads/muryeok.txt").unwrap();
        //let mut f = std::fs::File::open("/Users/rick/Downloads/linkor_test.txt").unwrap();
        let mut test_content: String = String::new();
        f.read_to_string(&mut test_content).unwrap();
        let test_content = iced::widget::text_editor::Content::with_text(&test_content);
        let app = App {
            args: args.clone(),
            test_content,
            font_loaded: true,
            char_gap: format!("{}", args.char_gap),
            jung_gap: format!("{}", args.jung_gap),
            jong_gap: format!("{}", args.jong_gap),
            sw_str: format!("{}", args.sw),
            text_size_str: "16".to_string(),
        };
        (app, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let args = &mut *ARGS.write().unwrap();
        match message {
            Message::Compile => {
                if std::path::Path::new(&mut self.args.source_filename).exists() {
                    self.args.char_gap = match self.char_gap.parse::<u16>() {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("Invalid char_gap: {}", self.char_gap);
                            100
                        }
                    };
                    self.args.jung_gap = match self.jung_gap.parse::<u16>() {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("Invalid glyph_gap: {}", self.jung_gap);
                            10
                        }
                    };
                    self.args.jong_gap = match self.jong_gap.parse::<u16>() {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("Invalid glyph_gap: {}", self.jong_gap);
                            10
                        }
                    };
                    compile(&self.args);
                }
                let mut f = std::fs::File::open(&self.args.target_filename).unwrap();
                let mut font_data: Vec<u8> = Vec::new();
                f.read_to_end(&mut font_data).unwrap();
                iced_graphics::text::font_system()
                    .write()
                    .unwrap()
                    .load_font(std::borrow::Cow::from(font_data));
                //let font = iced::font::load(font_data).map(Message::FontLoaded);
                //font
                self.font_loaded = !self.font_loaded;
                //*self = App::new().0;
                Task::none()
            }
            Message::CharGap(v) => {
                match v.parse::<u16>() {
                    Ok(v) => {
                        args.char_gap = v;
                    }
                    Err(_) => {}
                }
                self.char_gap = v;
                Task::none()
            }
            Message::JungGap(v) => {
                match v.parse::<u16>() {
                    Ok(v) => {
                        args.jung_gap = v;
                    }
                    Err(_) => {}
                }
                self.jung_gap = v;
                Task::none()
            }
            Message::JongGap(v) => {
                match v.parse::<u16>() {
                    Ok(v) => {
                        args.jong_gap = v;
                    }
                    Err(_) => {}
                }
                self.jong_gap = v;
                Task::none()
            }
            Message::TextSize(v) => {
                match v.parse::<u16>() {
                    Ok(v) => {
                        if v > 0 {
                            self.args.text_size = v;
                        }
                    }
                    Err(_) => {}
                }
                self.text_size_str = v;
                Task::none()
            }
            Message::StrokeWidth(v) => {
                match v.parse::<i16>() {
                    Ok(v) => {
                        if v > 0 {
                            self.args.sw = v;
                        }
                    }
                    Err(_) => {}
                }
                self.sw_str = v;
                Task::none()
            }
            Message::ChangeTtfFile(s) => {
                self.args.source_filename = s;
                Task::none()
            }
            Message::Edit(action) => {
                self.test_content.perform(action);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::widget::Column<Message> {
        let filename_text: iced::widget::TextInput<Message> =
            iced::widget::text_input("TTF file path", &self.args.source_filename)
                .id("input_filename")
                .on_input(Message::ChangeTtfFile)
                .into();
        let char_gap: iced::widget::TextInput<Message> =
            iced::widget::text_input("Character gap", &self.char_gap)
                .id("char_gap")
                .on_input(Message::CharGap)
                .into();
        let jung_gap: iced::widget::TextInput<Message> =
            iced::widget::text_input("Glyph gap", &self.jung_gap)
                .id("glyph_gap")
                .on_input(Message::JungGap)
                .into();
        let jong_gap: iced::widget::TextInput<Message> =
            iced::widget::text_input("Glyph gap", &self.jong_gap)
                .id("glyph_gap")
                .on_input(Message::JongGap)
                .into();
        let text_size: iced::widget::TextInput<Message> =
            iced::widget::text_input("Text size", &self.text_size_str)
                .id("text_size")
                .on_input(Message::TextSize)
                .into();
        let stroke_width: iced::widget::TextInput<Message> =
            iced::widget::text_input("Text size", &self.sw_str)
                .id("stroke_width")
                .on_input(Message::StrokeWidth)
                .into();
        let button_compile = iced::widget::button("Compile").on_press(Message::Compile);
        let row1 = iced::widget::row![
            filename_text,
            stroke_width,
            char_gap,
            jung_gap,
            jong_gap,
            text_size,
            button_compile
        ];
        let text = if self.font_loaded {
            iced::widget::text_editor(&self.test_content)
                .font(iced::Font::with_name("Linear Korean"))
                .size(self.args.text_size)
                .on_action(Message::Edit)
        } else {
            iced::widget::text_editor(&self.test_content)
                .font(iced::Font::with_name("PilGi"))
                .size(self.args.text_size)
                .on_action(Message::Edit)
        };
        let interface = iced::widget::column![row1, text];
        interface
    }
}

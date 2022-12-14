use std::{
    borrow::Cow,
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
};

use crate::*;
use chrono::{DateTime, Local};
use eframe;
use egui::{self, style::Margin, Color32, Frame, Id, Rounding, Stroke, WidgetText};
use home;
use rfd;

mod data;
mod increment_decrement;
mod remove;
mod valid_text;

use data::*;
use increment_decrement::{Arrows, Increment};

/*
let num_less_than_ten = ValText::with_validator(|text| {
  text.parse().ok().filter(|&n| n < 10)
});

ui.text_edit_singleline(&mut num_less_than_ten); */

const FILES_HEIGHT: f32 = 300.0;
const FRAME_RADIUS: f32 = 10.0;
const FRAME_MARGIN: f32 = 5.0;
const NUM_WIDTH: f32 = 15.0;

#[derive(Default)]
pub struct App<'a> {
    cwd: String,
    cwd_path: PathBuf,
    files: Vec<FileListing>,
    columns: (Columns, Order, Columns), // 3rd field is previous
    _add: AddData,
    case: CaseData,
    _date: DateData<'a>,
    extension: ExtensionData,
    folder: FolderData,
    name: NameData,
    _number: Numberdata,
    reg_exp: RegExData,
    remove: remove::RemoveData,
    replace: ReplaceData,
}

#[allow(clippy::from_over_into)]
impl Into<WidgetText> for &RenameFile {
    fn into(self) -> WidgetText {
        WidgetText::RichText(egui::widget_text::RichText::new(match &self.extension {
            None => self.stem.clone(),
            Some(ext) => format!("{}.{}", &self.stem, ext),
        }))
    }
}

/// Return the datetime as a localized date and time.
fn datetime_to_string(datetime: &DateTime<Local>) -> String {
    format!("{}", datetime.format("%x %X"))
}

/// Show just the filename for a file
fn file_no_parents(path: &Path) -> Cow<'_, str> {
    match path.file_name() {
        None => Cow::Owned(String::new()),
        Some(file) => match path.is_dir() {
            false => file.to_string_lossy(),
            true => {
                let mut folder = String::from("🗀");
                folder.push_str(&file.to_string_lossy());
                Cow::Owned(folder)
            }
        },
    }
}

/// Custom ordering for files. Directories at the start or end.
fn cmp(rhs: &Path, lhs: &Path) -> Ordering {
    match (rhs.is_dir(), lhs.is_dir()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => rhs.cmp(lhs),
    }
}

struct FileListing {
    name: PathBuf,
    renamed: RenameFile,
    extension: Option<String>,
    size: Option<u64>,
    modified: Option<DateTime<Local>>,
    created: Option<DateTime<Local>>,
    selected: bool,
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Columns {
    #[default]
    Name,
    NewName,
    Extension,
    Size,
    Created,
    Modified,
}
#[derive(Debug, Default)]
enum Order {
    #[default]
    Forward,
    Reverse,
}

impl App<'_> {
    //! Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut app: App = Default::default();
        let cwd_path = match home::home_dir() {
            Some(dir) => dir,
            None => PathBuf::default(),
        };
        app.cwd_path = cwd_path.clone();
        app.cwd = cwd_path.display().to_string();
        app.file_list();
        app
    }
    fn change_dir(&mut self) {
        self.cwd_path = PathBuf::from(&self.cwd);
        self.file_list();
    }

    fn up_one(&mut self) {
        if let Some(dir) = self.cwd_path.parent() {
            self.cwd_path = PathBuf::from(dir);
            self.cwd = self.cwd_path.display().to_string();
            self.file_list();
        };
    }

    fn file_list(&mut self) {
        if let Ok(dir) = self.cwd_path.read_dir() {
            let mut file_listing = Vec::new();
            for file in dir.flatten() {
                let name = file.path();
                let extension = name
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string());
                let renamed = RenameFile::new(&file.path());
                let mut size = None;
                let mut modified = None;
                let mut created = None;
                if let Ok(meta) = fs::metadata(&file.path()) {
                    #[cfg(windows)]
                    if format!("{:?}", meta.file_type()).contains("attributes: 38") {
                        continue; // Remove system hidden files (.blf, .regtrans-ms, etc)
                    }
                    if name.is_file() {
                        size = Some(meta.len())
                    };
                    if let Ok(dt) = meta.modified() {
                        modified = Some(dt.into());
                    };
                    if let Ok(dt) = meta.created() {
                        created = Some(dt.into());
                    };
                }
                if let Some(renamed) = renamed {
                    file_listing.push(FileListing {
                        name,
                        renamed,
                        extension,
                        size,
                        modified,
                        created,
                        selected: false,
                    });
                }
            }
            file_listing.sort_unstable_by(|lhs, rhs| cmp(&lhs.name, &rhs.name));
            self.files = file_listing;
        }
    }

    fn _process_selected(&mut self) {
        for (_cnt, file) in self.files.iter().enumerate() {
            if file.selected {
                let mut _orig = &file.name;
                let mut _renamed = &file.renamed;
                // self.add.make_options().process(&mut renamed);
            }
        }
    }
}

impl eframe::App for App<'_> {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let Self { label, value } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // Status bar.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Status: Ready");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    if ui.small_button("Select Folder").clicked() {
                        if let Some(dir) = rfd::FileDialog::new()
                            .set_directory(&self.cwd_path)
                            .pick_folder()
                        {
                            self.cwd = dir.display().to_string();
                            self.change_dir();
                        }
                    };
                    if ui.small_button("up").clicked() {
                        self.up_one();
                    };
                    let response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut self.cwd),
                    );
                    if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                        self.change_dir()
                    };
                });
                egui::ScrollArea::vertical()
                    .max_height(FILES_HEIGHT)
                    .show(ui, |ui| {
                        egui::Grid::new("Files").striped(true).show(ui, |ui| {
                            ui.label("Sel");
                            if ui
                                .selectable_value(&mut self.columns.0, Columns::Name, "Name")
                                .clicked()
                            {
                                match self.columns {
                                    (_, Order::Forward, Columns::Name) => {
                                        self.files
                                            .sort_unstable_by(|lhs, rhs| cmp(&rhs.name, &lhs.name));
                                        self.columns.1 = Order::Reverse;
                                    }
                                    _ => {
                                        self.files
                                            .sort_unstable_by(|lhs, rhs| cmp(&lhs.name, &rhs.name));
                                        self.columns.1 = Order::Forward;
                                    }
                                };
                                self.columns.2 = Columns::Name;
                            };
                            if ui
                                .selectable_value(&mut self.columns.0, Columns::NewName, "New Name")
                                .clicked()
                            {
                                match self.columns {
                                    (_, Order::Forward, Columns::NewName) => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            rhs.renamed.cmp(&lhs.renamed)
                                        });
                                        self.columns.1 = Order::Reverse;
                                    }
                                    _ => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            lhs.renamed.cmp(&rhs.renamed)
                                        });
                                        self.columns.1 = Order::Forward;
                                    }
                                };
                                self.columns.2 = Columns::NewName;
                            };
                            if ui
                                .selectable_value(&mut self.columns.0, Columns::Extension, "Type")
                                .clicked()
                            {
                                match self.columns {
                                    (_, Order::Forward, Columns::Extension) => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            rhs.extension.cmp(&lhs.extension)
                                        });
                                        self.columns.1 = Order::Reverse;
                                    }
                                    _ => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            lhs.extension.cmp(&rhs.extension)
                                        });
                                        self.columns.1 = Order::Forward;
                                    }
                                };
                                self.columns.2 = Columns::Extension;
                            };
                            if ui
                                .selectable_value(&mut self.columns.0, Columns::Size, "Size")
                                .clicked()
                            {
                                match self.columns {
                                    (_, Order::Forward, Columns::Size) => {
                                        self.files
                                            .sort_unstable_by(|lhs, rhs| rhs.size.cmp(&lhs.size));
                                        self.columns.1 = Order::Reverse;
                                    }
                                    _ => {
                                        self.files
                                            .sort_unstable_by(|lhs, rhs| lhs.size.cmp(&rhs.size));
                                        self.columns.1 = Order::Forward;
                                    }
                                };
                                self.columns.2 = Columns::Size;
                            };
                            if ui
                                .selectable_value(
                                    &mut self.columns.0,
                                    Columns::Modified,
                                    "Modified",
                                )
                                .clicked()
                            {
                                match self.columns {
                                    (_, Order::Forward, Columns::Modified) => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            rhs.modified.cmp(&lhs.modified)
                                        });
                                        self.columns.1 = Order::Reverse;
                                    }
                                    _ => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            lhs.modified.cmp(&rhs.modified)
                                        });
                                        self.columns.1 = Order::Forward;
                                    }
                                };
                                self.columns.2 = Columns::Modified;
                            };
                            if ui
                                .selectable_value(&mut self.columns.0, Columns::Created, "Created")
                                .clicked()
                            {
                                match self.columns {
                                    (_, Order::Forward, Columns::Created) => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            rhs.created.cmp(&lhs.created)
                                        });
                                        self.columns.1 = Order::Reverse;
                                    }
                                    _ => {
                                        self.files.sort_unstable_by(|lhs, rhs| {
                                            lhs.created.cmp(&rhs.created)
                                        });
                                        self.columns.1 = Order::Forward;
                                    }
                                };
                                self.columns.2 = Columns::Created;
                            };
                            ui.end_row();

                            for item in self.files.iter_mut() {
                                ui.checkbox(&mut item.selected, "");
                                ui.label(file_no_parents(&item.name));
                                ui.label(&item.renamed);
                                ui.label(if let Some(ext) = &item.extension {
                                    ext.as_str()
                                } else {
                                    ""
                                });
                                ui.label(if let Some(size) = &item.size {
                                    format!("{}", &size)
                                } else {
                                    String::new()
                                });
                                if let Some(time) = &item.modified {
                                    ui.label(datetime_to_string(time));
                                }
                                if let Some(time) = &item.created {
                                    ui.label(datetime_to_string(time));
                                }
                                ui.end_row();
                            }
                        });
                    });
                ui.horizontal(|ui| {
                    // ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center),
                    ui.vertical(|ui| {
                        Frame::none()
                            .stroke(Stroke::new(1.0, Color32::BLACK))
                            .inner_margin(Margin::same(FRAME_MARGIN))
                            .rounding(Rounding::same(FRAME_RADIUS))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label("Regex");
                                    ui.horizontal(|ui| {
                                        ui.label("Match:");
                                        ui.text_edit_singleline(&mut self.reg_exp.exp)
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Replacement:");
                                        ui.text_edit_singleline(&mut self.reg_exp.replace)
                                    });
                                    ui.checkbox(&mut self.reg_exp.extension, "Include Extension");
                                })
                            });
                        Frame::none()
                            .stroke(Stroke::new(1.0, Color32::BLACK))
                            .inner_margin(Margin::same(FRAME_MARGIN))
                            .rounding(Rounding::same(FRAME_RADIUS))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label("Name");
                                    egui::ComboBox::new("Name Options", "")
                                        .selected_text(&self.name.value)
                                        .show_ui(ui, |ui| {
                                            for opt in NameOpts::iterator() {
                                                ui.selectable_value(
                                                    &mut self.name.value,
                                                    opt,
                                                    format!("{:?}", opt),
                                                );
                                            }
                                        });
                                    ui.text_edit_singleline(&mut self.name.new);
                                })
                            });
                        Frame::none()
                            .stroke(Stroke::new(1.0, Color32::BLACK))
                            .inner_margin(Margin::same(FRAME_MARGIN))
                            .rounding(Rounding::same(FRAME_RADIUS))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label("Append Folder Name");
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::new("Append File Name", "")
                                            .selected_text(format!("{:?}", &self.folder.position))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.folder.position,
                                                    FolderMode::None,
                                                    "None",
                                                );
                                                ui.selectable_value(
                                                    &mut self.folder.position,
                                                    FolderMode::Prefix,
                                                    "Prefix",
                                                );
                                                ui.selectable_value(
                                                    &mut self.folder.position,
                                                    FolderMode::Suffix,
                                                    "Suffix",
                                                )
                                            });
                                        ui.label("Sep.");
                                        ui.text_edit_singleline(&mut self.folder.sep);
                                        ui.separator();
                                        ui.label("Pos.");
                                        if ui
                                            .add(
                                                egui::TextEdit::singleline(&mut self.folder.levels)
                                                    .desired_width(NUM_WIDTH),
                                            )
                                            .changed()
                                        {
                                            if !self.folder.levels.is_valid() {
                                                let prev = match self.folder.levels.get_prev() {
                                                    Some(v) => v,
                                                    None => 0,
                                                };
                                                self.folder.levels.set_val(prev);
                                            }
                                        };
                                        ui.add(Arrows {
                                            id: Id::new("Folder Arrows"),
                                            value: &mut self.folder,
                                            field: "folder",
                                        });
                                    });
                                });
                            });
                    });
                    ui.vertical(|ui| {
                        Frame::none()
                            .stroke(Stroke::new(1.0, Color32::BLACK))
                            .inner_margin(Margin::same(FRAME_MARGIN))
                            .rounding(Rounding::same(FRAME_RADIUS))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label("Replace");
                                    ui.horizontal(|ui| {
                                        ui.label("Replace: ");
                                        ui.text_edit_singleline(&mut self.replace.replace);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("With: ");
                                        ui.text_edit_singleline(&mut self.replace.with);
                                    });
                                    ui.checkbox(&mut self.replace.match_case, "Match Case")
                                });
                            });
                        Frame::none()
                            .stroke(Stroke::new(1.0, Color32::BLACK))
                            .inner_margin(Margin::same(FRAME_MARGIN))
                            .rounding(Rounding::same(FRAME_RADIUS))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label("Case");
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::new("Case", "")
                                            .selected_text(format!("{:?}", &self.case.choice))
                                            .show_ui(ui, |ui| {
                                                for opt in Case::iterator() {
                                                    ui.selectable_value(
                                                        &mut self.case.choice,
                                                        opt,
                                                        format!("{:?}", opt),
                                                    );
                                                }
                                            });
                                        ui.checkbox(&mut self.case.snake, "Snake_Case")
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Except:");
                                        ui.text_edit_singleline(&mut self.case.exceptions);
                                    });
                                })
                            });
                        Frame::none()
                            .stroke(Stroke::new(1.0, Color32::BLACK))
                            .inner_margin(Margin::same(FRAME_MARGIN))
                            .rounding(Rounding::same(FRAME_RADIUS))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.label("Extension");
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::new("Extension", "")
                                            .selected_text(format!("{:?}", &self.extension.value))
                                            .show_ui(ui, |ui| {
                                                for opt in ExtOpts::iterator() {
                                                    ui.selectable_value(
                                                        &mut self.extension.value,
                                                        opt,
                                                        format!("{:?}", opt),
                                                    );
                                                }
                                            });
                                        ui.text_edit_singleline(&mut self.extension.new);
                                    });
                                });
                            });
                    });
                    Frame::none()
                        .stroke(Stroke::new(1.0, Color32::BLACK))
                        .inner_margin(Margin::same(FRAME_MARGIN))
                        .rounding(Rounding::same(FRAME_RADIUS))
                        .show(ui, |ui| ui.add(remove::RemoveView::new(&mut self.remove)));
                    //     ui.vertical(|ui| {
                    //         ui.label("Remove");
                    //         ui.horizontal(|ui| {
                    //             ui.label("First n");
                    //             if ui
                    //                 .add(
                    //                     egui::TextEdit::singleline(&mut self.remove.first_n)
                    //                         .desired_width(NUM_WIDTH),
                    //                 )
                    //                 .changed()
                    //             {
                    //                 if !self.remove.first_n.is_valid() {
                    //                     let prev = match self.remove.first_n.get_prev() {
                    //                         Some(v) => v,
                    //                         None => 0,
                    //                     };
                    //                     self.remove.first_n.set_val(prev);
                    //                 }
                    //             };
                    //             ui.add(Arrows {
                    //                 id: Id::new("Remove First N"),
                    //                 value: &mut self.remove,
                    //                 field: "first_n",
                    //             });
                    //             ui.label("Last n");
                    //             if ui
                    //                 .add(
                    //                     egui::TextEdit::singleline(&mut self.remove.last_n)
                    //                         .desired_width(NUM_WIDTH),
                    //                 )
                    //                 .changed()
                    //             {
                    //                 if !self.remove.last_n.is_valid() {
                    //                     let prev = match self.remove.last_n.get_prev() {
                    //                         Some(v) => v,
                    //                         None => 0,
                    //                     };
                    //                     self.remove.last_n.set_val(prev);
                    //                 }
                    //             };
                    //             ui.add(Arrows {
                    //                 id: Id::new("Remove Last N"),
                    //                 value: &mut self.remove,
                    //                 field: "last_n",
                    //             });
                    //         });
                    //         ui.horizontal(|ui| {
                    //             ui.label("Start");
                    //             if ui
                    //                 .add(
                    //                     egui::TextEdit::singleline(&mut self.remove.start)
                    //                         .desired_width(NUM_WIDTH),
                    //                 )
                    //                 .changed()
                    //             {
                    //                 if !self.remove.start.is_valid() {
                    //                     let prev = match self.remove.start.get_prev() {
                    //                         Some(v) => v,
                    //                         None => 0,
                    //                     };
                    //                     self.remove.start.set_val(prev);
                    //                 }
                    //             };
                    //             ui.add(Arrows {
                    //                 id: Id::new("Start"),
                    //                 value: &mut self.remove,
                    //                 field: "start",
                    //             });
                    //             ui.label("End");
                    //             if ui
                    //                 .add(
                    //                     egui::TextEdit::singleline(&mut self.remove.end)
                    //                         .desired_width(NUM_WIDTH),
                    //                 )
                    //                 .changed()
                    //             {
                    //                 if !self.remove.end.is_valid() {
                    //                     let prev = match self.remove.end.get_prev() {
                    //                         Some(v) => v,
                    //                         None => 0,
                    //                     };
                    //                     self.remove.end.set_val(prev);
                    //                 }
                    //             };
                    //             ui.add(Arrows {
                    //                 id: Id::new("End"),
                    //                 value: &mut self.remove,
                    //                 field: "end",
                    //             });
                    //         });
                    //     });
                    // });
                    Frame::none()
                        .stroke(Stroke::new(1.0, Color32::BLACK))
                        .inner_margin(Margin::same(FRAME_MARGIN))
                        .rounding(Rounding::same(FRAME_RADIUS))
                        .show(ui, |ui| {
                            ui.label("Add");
                        });
                    Frame::none()
                        .stroke(Stroke::new(1.0, Color32::BLACK))
                        .inner_margin(Margin::same(FRAME_MARGIN))
                        .rounding(Rounding::same(FRAME_RADIUS))
                        .show(ui, |ui| {
                            ui.label("Auto Date");
                        });
                    Frame::none()
                        .stroke(Stroke::new(1.0, Color32::BLACK))
                        .inner_margin(Margin::same(FRAME_MARGIN))
                        .rounding(Rounding::same(FRAME_RADIUS))
                        .show(ui, |ui| {
                            ui.label("Numbering");
                        });
                });
            })
        });
    }
}

use seed::{prelude::*, *};

use crate::shared::styled_button::StyledButton;
use crate::shared::styled_icon::{Icon, StyledIcon};
use crate::shared::styled_tooltip::StyledTooltip;
use crate::shared::ToNode;
use crate::{FieldId, Msg};

#[derive(Debug, Clone, Copy)]
pub enum HeadingSize {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl ToString for HeadingSize {
    fn to_string(&self) -> String {
        match self {
            HeadingSize::H1 => "H1",
            HeadingSize::H2 => "H2",
            HeadingSize::H3 => "H3",
            HeadingSize::H4 => "H4",
            HeadingSize::H5 => "H5",
            HeadingSize::H6 => "H6",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub enum RteMsg {
    Bold,
    Italic,
    Underscore,
    Undo,
    Redo,
    Strikethrough,
    Copy,
    Paste,
    Cut,
    JustifyFull,
    JustifyCenter,
    JustifyLeft,
    JustifyRight,
    InsertParagraph,
    InsertHeading(HeadingSize),
    InsertUnorderedList,
    InsertOrderedList,
    RemoveFormat,
    Subscript,
    Superscript,
    TableSetRows(u16),
    TableSetColumns(u16),
    TableSetVisibility(bool),
    InsertTable { rows: u16, cols: u16 },
}

#[derive(Debug)]
pub struct ExecCommand {
    pub(crate) name: String,
    pub(crate) param: String,
}

impl ExecCommand {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self::new_with_param(name, "")
    }

    pub fn new_with_param<S1, S2>(name: S1, param: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            name: name.into(),
            param: param.into(),
        }
    }
}

impl RteMsg {
    pub fn to_command(&self) -> Option<ExecCommand> {
        match self {
            RteMsg::Bold => Some(ExecCommand::new("bold")),
            RteMsg::Italic => Some(ExecCommand::new("italic")),
            RteMsg::Underscore => Some(ExecCommand::new("underline")),
            RteMsg::Undo => Some(ExecCommand::new("undo")),
            RteMsg::Redo => Some(ExecCommand::new("redo")),
            RteMsg::Strikethrough => Some(ExecCommand::new("strikeThrough")),
            RteMsg::Copy => Some(ExecCommand::new("copy")),
            RteMsg::Paste => Some(ExecCommand::new("paste")),
            RteMsg::Cut => Some(ExecCommand::new("cut")),
            RteMsg::JustifyFull => Some(ExecCommand::new("justifyFull")),
            RteMsg::JustifyCenter => Some(ExecCommand::new("justifyCenter")),
            RteMsg::JustifyLeft => Some(ExecCommand::new("justifyLeft")),
            RteMsg::JustifyRight => Some(ExecCommand::new("justifyRight")),
            RteMsg::InsertParagraph => Some(ExecCommand::new("insertParagraph")),
            RteMsg::InsertHeading(heading) => {
                Some(ExecCommand::new_with_param("heading", heading.to_string()))
            }
            RteMsg::InsertUnorderedList => Some(ExecCommand::new("insertUnorderedList")),
            RteMsg::InsertOrderedList => Some(ExecCommand::new("insertOrderedList")),
            RteMsg::RemoveFormat => Some(ExecCommand::new("removeFormat")),
            RteMsg::Subscript => Some(ExecCommand::new("subscript")),
            RteMsg::Superscript => Some(ExecCommand::new("superscript")),
            RteMsg::InsertTable { .. } => None,
            // outer
            RteMsg::TableSetColumns(..)
            | RteMsg::TableSetRows(..)
            | RteMsg::TableSetVisibility(..) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StyledRteTableState {
    pub visible: bool,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug)]
pub struct StyledRteState {
    pub value: String,
    pub field_id: FieldId,
    pub table_tooltip: StyledRteTableState,
    range: Option<web_sys::Range>,
}

impl StyledRteState {
    pub fn new(field_id: FieldId) -> Self {
        Self {
            field_id,
            value: String::new(),
            table_tooltip: StyledRteTableState {
                visible: false,
                rows: 3,
                cols: 3,
            },
            range: None,
        }
    }

    pub fn update(&mut self, msg: &Msg) {
        let m = match msg {
            Msg::Rte(m, field) if field == &self.field_id => m,
            _ => return,
        };
        match m.to_command() {
            Some(ExecCommand { name, param }) => {
                self.store_range();
                match seed::html_document().exec_command_with_show_ui_and_value(
                    name.as_str(),
                    false,
                    param.as_str(),
                ) {
                    Ok(_) => {}
                    Err(e) => log!(e),
                }
                if self.restore_range().is_err() {
                    return;
                }
            }
            _ => match m {
                RteMsg::TableSetRows(n) => {
                    self.table_tooltip.rows = *n;
                }
                RteMsg::TableSetColumns(n) => {
                    self.table_tooltip.cols = *n;
                }
                RteMsg::TableSetVisibility(b) => {
                    if *b {
                        self.store_range();
                    }
                    self.table_tooltip.visible = *b;
                }
                RteMsg::InsertTable { rows, cols } => {
                    self.table_tooltip.visible = false;
                    self.table_tooltip.cols = 3;
                    self.table_tooltip.rows = 3;
                    if self.restore_range().is_err() {
                        return;
                    }
                    let doc = seed::html_document();
                    let r = match self.range.as_ref() {
                        Some(r) => r,
                        _ => return,
                    };
                    let table = match doc.create_element("table") {
                        Ok(t) => t,
                        _ => return,
                    };
                    let mut buff = "<tbody>".to_string();
                    for _c in 0..(*cols) {
                        buff.push_str("<tr>");
                        for _r in 0..(*rows) {
                            buff.push_str("<td>&nbsp;</td>")
                        }
                        buff.push_str("</tr>");
                    }
                    buff.push_str("</tbody>");
                    table.set_inner_html(buff.as_str());
                    if let Err(e) = r.insert_node(&table) {
                        log!(e);
                    }
                }
                _ => log!(m),
            },
        };
    }

    fn store_range(&mut self) {
        self.range = seed::html_document()
            .get_selection()
            .ok()
            .unwrap_or_else(|| None)
            .and_then(|s| s.get_range_at(0).ok());
    }

    fn restore_range(&mut self) -> Result<(), String> {
        let doc = seed::html_document();
        let sel = doc
            .get_selection()
            .ok()
            .unwrap_or_else(|| None)
            .ok_or_else(|| "Restoring selection failed. Unable to obtain select".to_string())?;
        let r = self
            .range
            .as_ref()
            .ok_or_else(|| "Restoring selection failed. No range was stored".to_string())?;
        sel.remove_all_ranges()
            .map_err(|_| "Restoring selection failed. Unable to remove ranges".to_string())?;
        sel.add_range(r).map_err(|_| {
            "Restoring selection failed. Unable to add current selection range".to_string()
        })?;
        Ok(())
    }
}

pub struct StyledRte {
    field_id: FieldId,
    table_tooltip: StyledRteTableState,
    // value: String,
}

impl StyledRte {
    pub fn build(field_id: FieldId) -> StyledRteBuilder {
        StyledRteBuilder {
            field_id,
            value: String::new(),
            table_tooltip: StyledRteTableState {
                visible: false,
                rows: 0,
                cols: 0,
            },
        }
    }
}

impl ToNode for StyledRte {
    fn into_node(self) -> Node<Msg> {
        render(self)
    }
}

pub struct StyledRteBuilder {
    field_id: FieldId,
    value: String,
    table_tooltip: StyledRteTableState,
}

impl StyledRteBuilder {
    pub fn state(mut self, state: &StyledRteState) -> Self {
        self.value = state.value.clone();
        self.table_tooltip = state.table_tooltip.clone();
        self
    }

    pub fn build(self) -> StyledRte {
        StyledRte {
            field_id: self.field_id,
            // value: self.value,
            table_tooltip: self.table_tooltip,
        }
    }
}

pub fn render(values: StyledRte) -> Node<Msg> {
    {
        let _brush_button = styled_rte_button(
            "Brush",
            Icon::Brush,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let _color_bucket_button = styled_rte_button(
            "Color bucket",
            Icon::ColorBucket,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let _color_picker_button = styled_rte_button(
            "Color picker",
            Icon::ColorPicker,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );

        let _link_broken_button = styled_rte_button(
            "Link broken",
            Icon::LinkBroken,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );

        let _pin_button = styled_rte_button(
            "Pin",
            Icon::Pin,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let _rotation_button = styled_rte_button(
            "Rotation",
            Icon::Rotation,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let _save_button = styled_rte_button(
            "Save",
            Icon::Save,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let _text_height_button = styled_rte_button(
            "Text height",
            Icon::TextHeight,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let _text_width_button = styled_rte_button(
            "Text width",
            Icon::TextWidth,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
    }

    let capture_event = ev(Ev::KeyDown, |ev| {
        ev.stop_propagation();
        None as Option<Msg>
    });
    let id = values.field_id.to_string();

    div![
        class!["styledRte"],
        attrs![At::Id => id],
        div![
            class!["bar"],
            first_row(&values),
            second_row(&values),
            // brush_button,
            // color_bucket_button,
            // color_picker_button,
            // link_broken_button,
            // pin_button,
            // save_button,
            // text_height_button,
            // text_width_button,
        ],
        div![
            class!["editorWrapper"],
            div![
                class!["editor"],
                attrs![At::ContentEditable => true],
                capture_event
            ],
        ]
    ]
}

fn first_row(values: &StyledRte) -> Node<Msg> {
    let justify = {
        let field_id = values.field_id.clone();
        let justify_all_button = styled_rte_button(
            "Justify All",
            Icon::JustifyAll,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();

                Some(Msg::Rte(RteMsg::JustifyFull, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let justify_center_button = styled_rte_button(
            "Justify Center",
            Icon::JustifyCenter,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::JustifyCenter, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let justify_left_button = styled_rte_button(
            "Justify Left",
            Icon::JustifyLeft,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::JustifyLeft, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let justify_right_button = styled_rte_button(
            "Justify Right",
            Icon::JustifyRight,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();

                Some(Msg::Rte(RteMsg::JustifyRight, field_id))
            }),
        );
        div![
            class!["group justify"],
            justify_all_button,
            justify_center_button,
            justify_left_button,
            justify_right_button
        ]
    };

    let system = {
        let field_id = values.field_id.clone();
        let redo_button = styled_rte_button(
            "Redo",
            Icon::Redo,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Redo, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let undo_button = styled_rte_button(
            "Undo",
            Icon::Undo,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Undo, field_id))
            }),
        );
        /*let field_id = values.field_id.clone();
        let clip_board_button = styled_rte_button(
            "Paste",
            Icon::ClipBoard,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Paste, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let copy_button = styled_rte_button(
            "Copy",
            Icon::Copy,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Copy, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let cut_button = styled_rte_button(
            "Cut",
            Icon::Cut,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Cut, field_id))
            }),
        );*/
        div![
            class!["group system"],
            // clip_board_button,
            // copy_button,
            // cut_button,
            undo_button,
            redo_button,
        ]
    };

    let formatting = {
        let field_id = values.field_id.clone();
        let remove_formatting = styled_rte_button(
            "Remove format",
            Icon::EraserAlt,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::RemoveFormat, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let bold_button = styled_rte_button(
            "Bold",
            Icon::Bold,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Bold, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let italic_button = styled_rte_button(
            "Italic",
            Icon::Italic,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Italic, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let underline_button = styled_rte_button(
            "Underline",
            Icon::Underline,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Underscore, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let strike_through_button = styled_rte_button(
            "StrikeThrough",
            Icon::StrikeThrough,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Strikethrough, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let subscript_button = styled_rte_button(
            "Subscript",
            Icon::Subscript,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Subscript, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let superscript_button = styled_rte_button(
            "Superscript",
            Icon::Superscript,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::Superscript, field_id))
            }),
        );
        div![
            class!["group formatting"],
            bold_button,
            italic_button,
            underline_button,
            strike_through_button,
            subscript_button,
            superscript_button,
            remove_formatting,
        ]
    };

    div![class!["row firstRow"], system, formatting, justify,]
}

fn second_row(values: &StyledRte) -> Node<Msg> {
    /*let align_group = {
        let field_id = values.field_id.clone();
        let align_center_button = styled_rte_button(
            "Align Center",
            Icon::AlignCenter,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let field_id = values.field_id.clone();
        let align_left_button = styled_rte_button(
            "Align Left",
            Icon::AlignLeft,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let field_id = values.field_id.clone();
        let align_right_button = styled_rte_button(
            "Align Right",
            Icon::AlignRight,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        div![
            class!["group align"],
            align_center_button,
            align_left_button,
            align_right_button,
        ]
    };*/

    let font_group = {
        let _field_id = values.field_id.clone();
        let _font_button = styled_rte_button(
            "Font",
            Icon::Font,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let options: Vec<Node<Msg>> = vec![
            HeadingSize::H1,
            HeadingSize::H2,
            HeadingSize::H3,
            HeadingSize::H4,
            HeadingSize::H5,
            HeadingSize::H6,
        ]
        .into_iter()
        .map(|h| {
            let field_id = values.field_id.clone();
            let button = StyledButton::build()
                .text(h.to_string())
                .on_click(mouse_ev(Ev::Click, move |ev| {
                    ev.prevent_default();
                    Some(Msg::Rte(RteMsg::InsertHeading(h), field_id))
                }))
                .empty()
                .build()
                .into_node();
            span![class!["headingOption"], button]
        })
        .collect();
        let heading_button = span![class!["headingList"], options];

        /*let _field_id = values.field_id.clone();
        let _small_cap_button = styled_rte_button(
            "Small Cap",
            Icon::SmallCap,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let _field_id = values.field_id.clone();
        let _all_caps_button = styled_rte_button(
            "All caps",
            Icon::AllCaps,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );*/
        div![
            class!["group font"],
            // font_button,
            heading_button,
            // small_cap_button,
            // all_caps_button
        ]
    };

    let insert_group = {
        let table_tooltip = table_tooltip(values);

        let field_id = values.field_id.clone();
        let listing_dots = styled_rte_button(
            "Listing dots",
            Icon::ListingDots,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::InsertUnorderedList, field_id))
            }),
        );
        let field_id = values.field_id.clone();
        let listing_number = styled_rte_button(
            "Listing number",
            Icon::ListingNumber,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::InsertOrderedList, field_id))
            }),
        );
        /*let field_id = values.field_id.clone();
        let sub_listing_button = styled_rte_button(
            "Sub Listing",
            Icon::SubListing,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );*/
        let field_id = values.field_id.clone();
        let mut table_button = styled_rte_button(
            "Table",
            Icon::Table,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::TableSetVisibility(true), field_id))
            }),
        );
        table_button.add_child(table_tooltip);

        let field_id = values.field_id.clone();
        let paragraph_button = styled_rte_button(
            "Paragraph",
            Icon::Paragraph,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                Some(Msg::Rte(RteMsg::InsertParagraph, field_id))
            }),
        );
        // let field_id = values.field_id.clone();
        // let code_alt_button = styled_rte_button(
        //     "Insert code",
        //     Icon::CodeAlt,
        //     mouse_ev(Ev::Click, move |ev| {
        //         ev.prevent_default();
        //         None as Option<Msg>
        //     }),
        // );

        div![
            class!["group insert"],
            paragraph_button,
            table_button,
            // code_alt_button,
            listing_dots,
            listing_number,
            // sub_listing_button,
        ]
    };

    let indent_outdent = {
        let indent_button = styled_rte_button(
            "Indent",
            Icon::Indent,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        let outdent_button = styled_rte_button(
            "Outdent",
            Icon::Outdent,
            mouse_ev(Ev::Click, move |ev| {
                ev.prevent_default();
                None as Option<Msg>
            }),
        );
        div![class!["group indentOutdent"], indent_button, outdent_button]
    };

    div![
        class!["row secondRow"],
        font_group,
        // align_group,
        insert_group,
        indent_outdent
    ]
}

fn table_tooltip(values: &StyledRte) -> Node<Msg> {
    let StyledRteTableState {
        visible,
        rows,
        cols,
    } = values.table_tooltip;
    let field_id = values.field_id.clone();
    let on_rows_change = input_ev(Ev::Change, move |v| {
        v.parse::<u16>()
            .ok()
            .map(|n| Msg::Rte(RteMsg::TableSetRows(n), field_id))
    });
    let field_id = values.field_id.clone();
    let on_cols_change = input_ev(Ev::Change, move |v| {
        v.parse::<u16>()
            .ok()
            .map(|n| Msg::Rte(RteMsg::TableSetColumns(n), field_id))
    });
    let field_id = values.field_id.clone();
    let close_table_tooltip = StyledButton::build()
        .empty()
        .icon(Icon::Close)
        .on_click(mouse_ev(Ev::Click, move |ev| {
            ev.prevent_default();
            Some(Msg::Rte(RteMsg::TableSetVisibility(false), field_id))
        }))
        .build()
        .into_node();
    let field_id = values.field_id.clone();
    let on_submit = mouse_ev(Ev::Click, move |ev| {
        ev.prevent_default();
        Some(Msg::Rte(RteMsg::InsertTable { rows, cols }, field_id))
    });
    StyledTooltip::build()
        .table_tooltip()
        .visible(visible)
        .add_child(h2![span!["Add table"], close_table_tooltip])
        .add_child(div![class!["inputs"], span!["Rows"], seed::input![
                attrs![At::Type => "range"; At::Step => "1"; At::Min => "1"; At::Max => "10"; At::Value => rows],
                on_rows_change
            ]])
        .add_child(div![
            class!["inputs"],
            span!["Columns"],
            seed::input![
                attrs![At::Type => "range"; At::Step => "1"; At::Min => "1"; At::Max => "10"; At::Value => cols],
                on_cols_change
            ]
        ])
        .add_child({
            let body: Vec<Node<Msg>> = (0..rows)
                .map(|_row| {
                    let tds: Vec<Node<Msg>> = (0..cols)
                        .map(|_col| td![" "])
                        .collect();
                    tr![tds]
                })
                .collect();
            seed::div![
                class!["tablePreview"],
                seed::table![tbody![body]],
                input![attrs![At::Type => "button"; At::Value => "Insert"], on_submit],
            ]
        })
        .build()
        .into_node()
}

fn styled_rte_button(title: &str, icon: Icon, handler: EventHandler<Msg>) -> Node<Msg> {
    let button = StyledButton::build()
        .icon(StyledIcon::build(icon).build())
        .on_click(handler)
        .empty()
        .build()
        .into_node();
    span![
        class!["styledRteButton"],
        attrs![At::Title => title],
        button
    ]
}

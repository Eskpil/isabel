use std::cell::RefCell;
use std::rc::Rc;

use crate::textmodel::{TextModel, TextRange};
use crate::Shell;
use serde::{Deserialize, Serialize};
use xkeysym::Keysym;

use crate::engine::Plugin;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "name")]
pub(super) enum TextInputType {
    #[serde(rename = "TextInputType.text")]
    Text,
    #[serde(rename = "TextInputType.multiline")]
    Multiline,
    #[serde(rename = "TextInputType.number")]
    Number { signed: bool, decimal: bool },
    #[serde(rename = "TextInputType.phone")]
    Phone,
    #[serde(rename = "TextInputType.datetime")]
    Datetime,
    #[serde(rename = "TextInputType.emailAddress")]
    EmailAddress,
    #[serde(rename = "TextInputType.url")]
    Url,
    #[serde(rename = "TextInputType.visiblePassword")]
    VisiblePassword,
    #[serde(rename = "TextInputType.name")]
    Name,
    #[serde(rename = "TextInputType.address")]
    StreetAddress,
    #[serde(rename = "TextInputType.none")]
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum BinaryType {
    #[serde(rename = "0")]
    Disabled,
    #[serde(rename = "1")]
    Enabled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum TextCapitalization {
    #[serde(rename = "TextCapitalization.words")]
    Words,
    #[serde(rename = "TextCapitalization.sentences")]
    Sentences,
    #[serde(rename = "TextCapitalization.characters")]
    Characters,
    #[serde(rename = "TextCapitalization.none")]
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum Brightness {
    #[serde(rename = "Brightness.dark")]
    Dark,
    #[serde(rename = "Brightness.light")]
    Light,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct TextClientParameters {
    pub(super) input_type: TextInputType,
    pub(super) read_only: bool,
    pub(super) obscure_text: bool,
    pub(super) autocorrect: bool,
    pub(super) smart_dashes_type: BinaryType,
    pub(super) smart_quotes_type: BinaryType,
    pub(super) enable_suggestions: bool,
    pub(super) enable_interactive_selection: bool,
    pub(super) action_label: Option<String>,
    pub(super) input_action: TextInputAction,
    pub(super) text_capitalization: TextCapitalization,
    pub(super) keyboard_appearance: Brightness,
    #[serde(rename = "enableIMEPersonalizedLearning")]
    pub(super) enable_ime_personalized_learning: bool,
    pub(super) enable_delta_model: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct SetEditableSizeAndTransformParameters {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) transform: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct GenericRectParameters {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) x: f32,
    pub(super) y: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct SetStyleParameters {
    pub(super) font_family: String,
    pub(super) font_size: f32,
    pub(super) font_weight_index: u32,
    pub(super) text_align_index: u32,
    pub(super) text_direction_index: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
pub(super) enum TextInput {
    #[serde(rename = "TextInput.setClient")]
    SetClient(u64, TextClientParameters),
    #[serde(rename = "TextInput.show")]
    Show,
    #[serde(rename = "TextInput.setEditingState")]
    SetEditingState(TextEditingValue),
    #[serde(rename = "TextInput.clearClient")]
    ClearClient,
    #[serde(rename = "TextInput.hide")]
    Hide,
    #[serde(rename = "TextInput.setEditableSizeAndTransform")]
    SetEditableSizeAndTransform(SetEditableSizeAndTransformParameters),
    #[serde(rename = "TextInput.setMarkedTextRect")]
    SetMarkedTextRect(GenericRectParameters),
    #[serde(rename = "TextInput.setStyle")]
    SetStyle(SetStyleParameters),
    #[serde(rename = "TextInput.requestAutofill")]
    RequestAutofill,
    #[serde(rename = "TextInput.setCaretRect")]
    SetCaretRect(GenericRectParameters),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum TextAffinity {
    #[serde(rename = "TextAffinity.downstream")]
    Downstream,
    #[serde(rename = "TextAffinity.upstream")]
    Upstream,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct TextEditingValue {
    pub(super) text: String,
    pub(super) selection_base: Option<i64>,
    pub(super) selection_extent: Option<i64>,
    pub(super) selection_affinity: Option<TextAffinity>,
    pub(super) selection_is_directional: Option<bool>,
    pub(super) composing_base: Option<i64>,
    pub(super) composing_extent: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(super) enum TextInputAction {
    #[serde(rename = "TextInputAction.none")]
    None,
    #[serde(rename = "TextInputAction.unspecified")]
    Unspecified,
    #[serde(rename = "TextInputAction.done")]
    Done,
    #[serde(rename = "TextInputAction.go")]
    Go,
    #[serde(rename = "TextInputAction.search")]
    Search,
    #[serde(rename = "TextInputAction.send")]
    Send,
    #[serde(rename = "TextInputAction.next")]
    Next,
    #[serde(rename = "TextInputAction.previous")]
    Previous,
    #[serde(rename = "TextInputAction.continueAction")]
    ContinueAction,
    #[serde(rename = "TextInputAction.join")]
    Join,
    #[serde(rename = "TextInputAction.route")]
    Route,
    #[serde(rename = "TextInputAction.emergencyCall")]
    EmergencyCall,
    #[serde(rename = "TextInputAction.newline")]
    Newline,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
pub(super) enum TextInputClient {
    #[serde(rename = "TextInputClient.updateEditingState")]
    UpdateEditingState(u64, TextEditingValue),
    #[serde(rename = "TextInputClient.updateEditingStateWithDeltas")]
    UpdateEditingWithDeltas(u64, serde_json::Map<String, serde_json::Value>),
    #[serde(rename = "TextInputClient.updateEditingStateWithTag")]
    UpdateEditingStateWithTag(u64, serde_json::Map<String, serde_json::Value>),
    #[serde(rename = "TextInputClient.performAction")]
    PerformAction(u64, TextInputAction),
    #[serde(rename = "TextInputClient.performSelectors")]
    PerformSelectors(u64, Vec<serde_json::Value>),
    #[serde(rename = "TextInputClient.requestExistingInputState")]
    RequestExistingInputState,
    #[serde(rename = "TextInputClient.updateFloatingCursor")]
    UpdateFloatingCursor(u64, String),
    #[serde(rename = "TextInputClient.onConnectionClosed")]
    OnConnectionClosed(u64),
    #[serde(rename = "TextInputClient.showAutocorrectionPromptRect")]
    ShowAutocorrectionPromptRect(u64),
    #[serde(rename = "TextInputClient.showToolbar")]
    ShowToolbar(u64),
    #[serde(rename = "TextInputClient.insertTextPlaceholder")]
    InsertTextPlaceholder(u64, f64, f64),
    #[serde(rename = "TextInputClient.removeTextPlaceholder")]
    RemoveTextPlaceholder(u64),
}

fn publish<T: Serialize>(channel: String, data: &T, shell: &mut Rc<RefCell<dyn Shell>>) {
    let payload = serde_json::ser::to_vec(data).expect("could not serialize");

    shell
        .borrow_mut()
        .instance_mut()
        .engine_mut()
        .publish(channel, payload);
}

#[derive(Clone)]
pub struct Textinput {
    shell: Option<Rc<RefCell<dyn Shell>>>,

    active_model: Option<TextModel>,
    active_client_id: Option<u64>,
    active_input_action: Option<TextInputAction>,
    active_input_type: Option<TextInputType>,
}

impl Textinput {
    pub fn new() -> Self {
        Self {
            shell: None,

            active_model: None,
            active_client_id: None,
            active_input_action: None,
            active_input_type: None,
        }
    }

    fn handle_enter_pressed(&mut self) {
        if self.active_model.is_none() {
            return;
        }

        if self.active_input_type == Some(TextInputType::Multiline) {
            self.active_model.as_mut().unwrap().add_codepoint('\n');
        }

        let method = TextInputClient::PerformAction(
            self.active_client_id.unwrap(),
            self.active_input_action.unwrap().clone(),
        );

        publish(self.on().to_owned(), &method, self.shell.as_mut().unwrap());
    }

    pub fn handle_key_event(&mut self, symbol: xkeysym::Keysym, _: bool) {
        if self.active_model.is_none() {
            return;
        }

        let mut changed = false;

        match symbol {
            Keysym::Left => {
                changed = self.active_model.as_mut().unwrap().move_cursor_back();
            }
            Keysym::Right => {
                changed = self.active_model.as_mut().unwrap().move_cursor_forward();
            }
            Keysym::BackSpace => {
                changed = self.active_model.as_mut().unwrap().backspace();
            }
            Keysym::Home => {
                changed = self
                    .active_model
                    .as_mut()
                    .unwrap()
                    .move_cursor_to_beginning();
            }
            Keysym::Return => {
                self.handle_enter_pressed();
            }
            _ => {
                if let Some(cp) = symbol.key_char() {
                    self.active_model.as_mut().unwrap().add_codepoint(cp);
                }
                changed = true;
            }
        }

        if !changed {
            return;
        }

        self.publish_state();
    }

    pub fn publish_state(&mut self) {
        if self.active_model.is_none() {
            return;
        }

        let mut args = TextEditingValue::default();

        args.composing_base = Some(-1);
        args.composing_extent = Some(-1);

        let selection_base = self.active_model.as_ref().unwrap().selection.start();
        let selection_extent = self.active_model.as_ref().unwrap().selection.end();

        args.selection_affinity = Some(TextAffinity::Downstream);
        args.selection_base = Some(selection_base as i64);
        args.selection_extent = Some(selection_extent as i64);

        args.selection_is_directional = Some(false);

        args.text = self.active_model.as_ref().unwrap().get_text();

        let cmd =
            TextInputClient::UpdateEditingState(*self.active_client_id.as_ref().unwrap(), args);
        publish(self.on().to_owned(), &cmd, self.shell.as_mut().unwrap());
    }
}

impl crate::engine::Plugin for Textinput {
    fn init(&mut self, shell: Rc<RefCell<dyn Shell>>) -> anyhow::Result<()> {
        self.shell = Some(shell);
        Ok(())
    }

    fn on(&self) -> &str {
        "flutter/textinput"
    }

    fn handle(&mut self, payload: Vec<u8>) -> anyhow::Result<()> {
        let textinput: TextInput = serde_json::from_slice(&payload[..])?;
        match textinput {
            TextInput::SetClient(id, client) => {
                self.active_model = Some(TextModel::new());
                self.active_client_id = Some(id);
                self.active_input_type = Some(client.input_type.clone());
                self.active_input_action = Some(client.input_action.clone());
            }
            TextInput::SetEditingState(args) => {
                if self.active_model.is_none() {
                    unreachable!("model should be active");
                }

                let mut base = args.selection_base.unwrap();
                let mut extent = args.selection_extent.unwrap();
                if base == -1 && extent == -1 {
                    base = 0;
                    extent = 0;
                }

                self.active_model.as_mut().unwrap().set_text(&args.text);
                self.active_model
                    .as_mut()
                    .unwrap()
                    .set_selection(TextRange::new(base as usize, extent as usize));
            }
            TextInput::ClearClient => {
                self.active_model = None;
                self.active_client_id = None;
                self.active_input_type = None;
                self.active_input_action = None;
            }

            o => log::debug!("implement: {:?}", o),
        }

        Ok(())
    }
}

//! This module implements the egui-node-graph traits and such for the project's System.

use egui_node_graph as eng;

use egui::Color32;
use eng::Node;

use crate::project::{
    Project,
    System,
};

use crate::board::{
    Board,
    pinout::Interface,
};

use crate::app;

impl eng::DataTypeTrait<System> for Interface {
    
    fn data_type_color(&self, user_state: &mut System) -> Color32 {
        // TODO - do this based on colorscheme?
        egui::Color32::WHITE
    }

    fn name(&self) -> std::borrow::Cow<str> {
        // let s = format!("{}", self).as_str().to_owned();
        // std::borrow::Cow::Borrowed(s)
        std::borrow::Cow::Borrowed("interface")
    }

}

struct BoardList(Vec<Board>);
impl eng::NodeTemplateIter for BoardList {
    type Item = Board;
    fn all_kinds(&self) -> Vec<Self::Item> {
        self.0.clone()
    }
}

#[derive(Debug, Clone)]
pub enum NodeResponseType {
    NONE
}
impl eng::UserResponseTrait for NodeResponseType {}
impl eng::NodeDataTrait for Board {
    type DataType = Interface;
    type Response = NodeResponseType; // not using this for now
    type UserState = System;
    type ValueType = Interface;

    fn bottom_ui(
            &self,
            ui: &mut egui::Ui,
            node_id: eng::NodeId,
            graph: &eng::Graph<Self, Self::DataType, Self::ValueType>,
            user_state: &mut Self::UserState,
        ) -> Vec<eng::NodeResponse<Self::Response, Self>>
        where
            Self::Response: eng::UserResponseTrait
    {
        ui.label("bottom");
        return vec![eng::NodeResponse::User(NodeResponseType::NONE)];
    }

}

impl eng::WidgetValueTrait for Interface {
    type Response = NodeResponseType;
    type NodeData = Board;
    type UserState = System;
    fn value_widget(
            &mut self,
            param_name: &str,
            node_id: eng::NodeId,
            ui: &mut egui::Ui,
            user_state: &mut Self::UserState,
            node_data: &Self::NodeData,
        ) -> Vec<Self::Response>
    {
        match self {
            Interface::GPIO => {
                ui.label("gpio interface");
            },
            _ => {
                ui.label("other interface");
            },
        }
        vec![]
    }
}

impl eng::NodeTemplateTrait for Board {
    type DataType = Interface;
    type NodeData = Board;
    type UserState = System;
    type ValueType = Interface;
    type CategoryType = &'static str;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        let name = self.get_name();
        std::borrow::Cow::Borrowed(name)
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<Self::CategoryType> {
        vec!["category1: boards"]
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        self.clone()
    }

    fn build_node(
            &self,
            graph: &mut eng::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
            user_state: &mut Self::UserState,
            node_id: eng::NodeId,
        )
    {
        match self.is_main_board() {
            true => {
                if self.pinout.len() > 0 {
                    graph.add_output_param(node_id, "out".to_string(), self.pinout[0].interface.clone());
                }
            },
            false => {
                if self.pinout.len() > 0 {
                    graph.add_input_param(node_id, "in".to_string(), self.pinout[0].interface.clone(), Interface::default(), eng::InputParamKind::ConnectionOnly, true);
                }
            }
        }
    }
}

type SystemGraph = eng::Graph<Board, Interface, Interface>;
pub type SystemEditorState =
    eng::GraphEditorState<Board, Interface, Interface, Board, System>;

impl Project {

    /// Display the node editor in the calling container.
    pub fn display_system_node_graph(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, known_boards: Vec<Board>) {
        let kb = BoardList(known_boards);
        self.graph_editor.draw_graph_editor(
            ui,
            kb,
            &mut self.system,
            Vec::default(),
        );
    }

}
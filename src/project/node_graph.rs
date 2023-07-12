//! This module implements the egui-node-graph traits and such for the project's System.

use log::{info, warn};

use egui_node_graph as eng;

use egui::Color32;

use crate::project::{
    Project,
    System,
};

use crate::board::{
    Board,
    BoardMiniWidget,
};

use crate::board::pinout::{
    InterfaceType,
    InterfaceDirection,
    Interface
};

impl eng::DataTypeTrait<System> for InterfaceType {
    
    fn data_type_color(&self, user_state: &mut System) -> Color32 {
        // TODO - do this based on colorscheme?
        match self {
            InterfaceType::I2C => egui::Color32::GREEN,
            InterfaceType::GPIO => egui::Color32::WHITE,
            InterfaceType::ADC => egui::Color32::BLUE,
            _ => egui::Color32::RED,
        }
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

/// This trait allows to add a custom Ui element to the bottom of the Node window
impl eng::NodeDataTrait for Board {
    type DataType = InterfaceType;
    type Response = NodeResponseType; // not using this for now
    type UserState = System;
    type ValueType = Interface;

    fn bottom_ui(
            &self,
            ui: &mut egui::Ui,
            _node_id: eng::NodeId,
            _graph: &eng::Graph<Self, Self::DataType, Self::ValueType>,
            _user_state: &mut Self::UserState,
        ) -> Vec<eng::NodeResponse<Self::Response, Self>>
        where
            Self::Response: eng::UserResponseTrait
    {
        ui.add(BoardMiniWidget(self.clone()));
        return vec![];
    }

}

/// This trait defines display aspects for each Node Template, for use in the
/// Node Finder, Node windows, etc
impl eng::NodeTemplateTrait for Board {
    type DataType = InterfaceType;
    type NodeData = Board;
    type UserState = System;
    type ValueType = Interface;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        let name = self.get_name();
        std::borrow::Cow::Borrowed(name)
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<Self::CategoryType> {
        vec![]
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        self.clone()
    }

    fn build_node(
            &self,
            graph: &mut eng::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
            _user_state: &mut Self::UserState,
            node_id: eng::NodeId,
        )
    {
        for po in self.pinout.iter() {
            match po.interface.direction {
                InterfaceDirection::Output => {
                    let name = format!("{} : {}", po.interface.iface_type.to_string(), po.interface.direction.to_string());
                    graph.add_output_param(node_id, name, po.interface.iface_type.clone());
                },
                InterfaceDirection::Input => {
                    let name = format!("{} : {}", po.interface.iface_type.to_string(), po.interface.direction.to_string());
                    graph.add_input_param(node_id, name, po.interface.iface_type.clone(), po.interface.clone(), eng::InputParamKind::ConnectionOnly, true);
                },
                _ => {
                    // graph.add_output_param(node_id, po.interface.clone().to_string(), po.interface.clone());
                    info!("found pinout interface that isn't implemented in the graph editor!")
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeResponseType {
    NONE
}
impl eng::UserResponseTrait for NodeResponseType {}

/// This trait specifies what *value* to draw in the Ui for each Interface in a Node.
/// For inputs this will be displayed after a connection is made.
impl eng::WidgetValueTrait for Interface {
    type Response = NodeResponseType;
    type NodeData = Board;
    type UserState = System;
    fn value_widget(
            &mut self,
            _param_name: &str,
            _node_id: eng::NodeId,
            ui: &mut egui::Ui,
            _user_state: &mut Self::UserState,
            _node_data: &Self::NodeData,
        ) -> Vec<Self::Response>
    {
        let name = format!("{} : {}", self.iface_type.to_string(), self.direction.to_string());
        ui.label(name);
        vec![]
    }
}

// type SystemGraph = eng::Graph<Board, Interface, Interface>;
pub type SystemEditorState =
    eng::GraphEditorState<Board, InterfaceType, Interface, Board, System>;

impl Project {

    /// Display the node editor in the calling container.
    pub fn display_system_node_graph(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, known_boards: Vec<Board>) {
        let kb = BoardList(known_boards);
        let gr: eng::GraphResponse<NodeResponseType, Board> = self.graph_editor.draw_graph_editor(
            ui,
            kb,
            &mut self.system,
            Vec::default(),
        );
        // look through the reponses, and perform appropriate actions
        gr.node_responses.iter().for_each(|response| {
            match response {
                eng::NodeResponse::DeleteNodeFull { node_id, node } => {
                    info!("removing node from system...");
                    match self.system.boards.iter().position(|elem| *elem == node.user_data) {
                        Some(idx) => {
                            self.system.boards.remove(idx);
                        },
                        None => {
                            warn!("deleting node: couldn't find board in system (this is probably a bug!");
                        }
                    }
                },
                _ => {
                    ()
                },
            }
        });

    }

}
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use crossterm::event::KeyCode;
use oombak_sim::sim::{InstanceNode, LoadedDut, Signal};
use ratatui::style::Color;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{palette::tailwind::SLATE, Modifier, Style},
    text::Line,
    widgets::{List, ListItem, ListState, Paragraph},
};

use crate::{
    component::{Component, HandleResult},
    render::Message,
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const INSTANCE_ITEM_STYLE: Style = Style::new()
    .fg(Color::Blue)
    .add_modifier(Modifier::UNDERLINED);
const SIGNAL_ITEM_STYLE: Style = Style::new()
    .fg(Color::Yellow)
    .add_modifier(Modifier::ITALIC);

pub struct InstanceHierViewer {
    message_tx: Sender<Message>,
    root_node: Option<Arc<RwLock<InstanceHierNode>>>,
    probed_points: HashSet<String>,
    items_in_list: Vec<HierItem>,
    list_state: ListState,
    selected_item_idx: Option<usize>,
    signals_marked_to_add: HashSet<String>,
    signals_marked_to_remove: HashSet<String>,
}

struct InstanceHierNode {
    path: String,
    module_name: String,
    children: Vec<Arc<RwLock<InstanceHierNode>>>,
    leafs: Vec<Arc<RwLock<InstanceHierLeaf>>>,
    is_expanded: bool,
}

#[derive(Clone)]
struct InstanceHierLeaf {
    path: String,
    signal: Signal,
    is_added: bool,
    marker: Marker,
}

#[derive(Clone)]
enum Marker {
    NotMarked,
    MarkedForAdd,
    MarkedForRemove,
}

enum HierItem {
    Instance(Arc<RwLock<InstanceHierNode>>),
    Signal(Arc<RwLock<InstanceHierLeaf>>),
}

impl InstanceHierViewer {
    pub fn new(message_tx: Sender<Message>) -> Self {
        Self {
            message_tx,
            root_node: None,
            items_in_list: vec![],
            list_state: ListState::default(),
            selected_item_idx: None,
            probed_points: HashSet::default(),
            signals_marked_to_add: HashSet::default(),
            signals_marked_to_remove: HashSet::default(),
        }
    }

    pub fn set_loaded_dut(&mut self, loaded_dut: &LoadedDut) {
        self.probed_points = HashSet::from_iter(loaded_dut.probed_points.iter().cloned());
        self.root_node = Some(Arc::new(RwLock::new(InstanceHierNode::new(
            &loaded_dut.root_node,
            "",
            &self.probed_points,
        ))));
        self.selected_item_idx = Some(0);
        self.list_state.select_first();
    }
}

impl Component for InstanceHierViewer {
    fn render_mut(&mut self, f: &mut ratatui::Frame, rect: ratatui::prelude::Rect) {
        if let Some(node) = &self.root_node {
            let (list_items, items_in_list) = Self::get_flattened_hierarchy(node);
            self.items_in_list = items_in_list;
            let list = List::new(list_items).highlight_style(SELECTED_STYLE);
            f.render_stateful_widget(list, rect, &mut self.list_state);
        } else {
            let rect = Layout::vertical(vec![
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ])
            .split(rect)[1];
            let message = Paragraph::new("DUT not loaded").alignment(Alignment::Center);
            f.render_widget(message, rect);
        }
    }

    fn handle_key_event(
        &mut self,
        key_event: &crossterm::event::KeyEvent,
    ) -> crate::component::HandleResult {
        match key_event.code {
            KeyCode::Char('q') => return HandleResult::ReleaseFocus,
            KeyCode::Enter => self.perform_action_on_selected(),
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(idx) = self.selected_item_idx {
                    self.list_state.select_next();
                    let new_idx = usize::saturating_add(idx, 1);
                    self.selected_item_idx =
                        Some(usize::min(self.items_in_list.len() - 1, new_idx));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(idx) = self.selected_item_idx {
                    self.list_state.select_previous();
                    self.selected_item_idx = Some(usize::saturating_sub(idx, 1));
                }
            }
            _ => (),
        }
        self.notify_render();
        HandleResult::Handled
    }

    fn try_propagate_event(
        &mut self,
        _event: &crossterm::event::Event,
    ) -> crate::component::HandleResult {
        HandleResult::NotHandled
    }

    fn set_focus_to_self(&mut self) {}

    fn render(&self, _f: &mut ratatui::Frame, _rect: ratatui::prelude::Rect) {
        todo!()
    }
}

impl InstanceHierViewer {
    fn notify_render(&self) {
        self.message_tx.send(Message::Render).unwrap();
    }

    fn get_flattened_hierarchy(
        node: &Arc<RwLock<InstanceHierNode>>,
    ) -> (Vec<ListItem<'static>>, Vec<HierItem>) {
        let mut list_items = vec![];
        let mut items_in_list = vec![];
        Self::traverse_hier_node(node, &mut list_items, &mut items_in_list, 0);
        (list_items, items_in_list)
    }

    fn traverse_hier_node(
        node: &Arc<RwLock<InstanceHierNode>>,
        list_items: &mut Vec<ListItem>,
        items_in_list: &mut Vec<HierItem>,
        depth: usize,
    ) {
        list_items.push(Self::new_instance_list_item(node, depth));
        items_in_list.push(HierItem::Instance(node.clone()));
        let node = node.read().unwrap();
        if node.is_expanded {
            for leaf in node.leafs.iter() {
                list_items.push(Self::new_signal_list_item(leaf, depth + 1));
                items_in_list.push(HierItem::Signal(leaf.clone()));
            }
            for node in node.children.iter() {
                Self::traverse_hier_node(node, list_items, items_in_list, depth + 1);
            }
        }
    }

    fn new_instance_list_item<'a>(
        node: &Arc<RwLock<InstanceHierNode>>,
        depth: usize,
    ) -> ListItem<'a> {
        let node = node.read().unwrap();
        let indentation = " ".repeat(depth * 2);
        let expand_or_collapse_symbol = if node.is_expanded { "[-]" } else { "[+]" };
        let line = Line::raw(format!(
            "{}{} {} ({})",
            indentation, expand_or_collapse_symbol, node.path, node.module_name
        ))
        .style(INSTANCE_ITEM_STYLE);
        ListItem::new(line)
    }

    fn new_signal_list_item<'a>(
        leaf: &Arc<RwLock<InstanceHierLeaf>>,
        depth: usize,
    ) -> ListItem<'a> {
        let leaf = leaf.read().unwrap();
        let indentation = " ".repeat(depth * 2);
        let added_symbol = if leaf.is_added { " (*)" } else { "" };
        let marker_symbol = match leaf.marker {
            Marker::NotMarked => "",
            Marker::MarkedForAdd => " (+)",
            Marker::MarkedForRemove => " (-)",
        };
        let line = Line::raw(format!(
            "{}{}{}{}",
            indentation, leaf.signal.name, added_symbol, marker_symbol
        ))
        .style(SIGNAL_ITEM_STYLE);
        ListItem::new(line)
    }

    fn perform_action_on_selected(&mut self) {
        let mut signals_marked_to_add = self.signals_marked_to_add.clone();
        let mut signals_marked_to_remove = self.signals_marked_to_remove.clone();
        if let Some(item) = self.get_selected_item() {
            match item {
                HierItem::Instance(node) => {
                    let mut node = node.write().unwrap();
                    node.is_expanded = !node.is_expanded;
                }
                HierItem::Signal(leaf) => {
                    let mut leaf = leaf.write().unwrap();
                    leaf.marker = match leaf.marker {
                        Marker::NotMarked if leaf.is_added => {
                            signals_marked_to_remove.insert(leaf.path.clone());
                            Marker::MarkedForRemove
                        }
                        Marker::NotMarked => {
                            signals_marked_to_add.insert(leaf.path.clone());
                            Marker::MarkedForAdd
                        }
                        Marker::MarkedForAdd => {
                            signals_marked_to_add.remove(&leaf.path);
                            Marker::NotMarked
                        }
                        Marker::MarkedForRemove => {
                            signals_marked_to_remove.remove(&leaf.path);
                            Marker::NotMarked
                        }
                    };
                }
            }
            self.notify_render();
        }
    }

    fn get_selected_item(&self) -> Option<&HierItem> {
        if let Some(idx) = self.selected_item_idx {
            Some(&self.items_in_list[idx])
        } else {
            None
        }
    }
}

impl InstanceHierNode {
    fn new(
        instance_node: &InstanceNode,
        parent_path: &str,
        probed_points: &HashSet<String>,
    ) -> Self {
        let path = format!("{parent_path}.{}", instance_node.name);
        let children: Vec<Arc<RwLock<InstanceHierNode>>> = instance_node
            .children
            .iter()
            .map(|n| InstanceHierNode::new(n, &path, probed_points))
            .map(RwLock::new)
            .map(Arc::new)
            .collect();
        let leafs = instance_node
            .signals
            .iter()
            .map(|s| InstanceHierLeaf::new(s, &path, probed_points))
            .map(RwLock::new)
            .map(Arc::new)
            .collect();
        InstanceHierNode {
            path,
            module_name: instance_node.module_name.clone(),
            leafs,
            is_expanded: false,
            children,
        }
    }
}

impl InstanceHierLeaf {
    fn new(signal: &Signal, parent_path: &str, probed_points: &HashSet<String>) -> Self {
        let path = format!("{parent_path}.{}", signal.name);
        let is_added = probed_points.contains(&path);
        InstanceHierLeaf {
            path,
            signal: signal.clone(),
            is_added,
            marker: Marker::NotMarked,
        }
    }
}